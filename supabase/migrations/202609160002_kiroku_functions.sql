-- Kiroku 云同步 · Migration 002：受控函数（客户端唯一访问面）
-- 对应 SYNC_DESIGN.md §4.2–§8。全部 SECURITY DEFINER + search_path = ''。
-- 约定：认证/参数违规用异常（PostgREST → 4xx）；可预期业务结果用返回 JSON 的 status 字段。

-- ---------- 内部工具 ----------

create or replace function public.kiroku_require_user()
returns uuid
language plpgsql stable security definer set search_path = '' as $$
declare v_uid uuid;
begin
    v_uid := auth.uid();
    if v_uid is null then
        raise insufficient_privilege using message = 'NOT_AUTHENTICATED';
    end if;
    return v_uid;
end;
$$;

-- 确保 sync_state 行存在并加行锁；同用户推送经此串行化
create or replace function public.kiroku_ensure_state(p_uid uuid)
returns public.kiroku_sync_state
language plpgsql volatile security definer set search_path = '' as $$
declare v_state public.kiroku_sync_state;
begin
    insert into public.kiroku_sync_state (user_id) values (p_uid)
    on conflict (user_id) do nothing;
    select * into v_state from public.kiroku_sync_state
    where user_id = p_uid for update;
    return v_state;
end;
$$;

-- 会员状态：active = 有效会员；retention = 到期后 90 天保留期内
create or replace function public.kiroku_membership_status(p_uid uuid)
returns table (has_row boolean, active boolean, in_retention boolean, expires_at timestamptz)
language plpgsql stable security definer set search_path = '' as $$
declare v_exp timestamptz;
begin
    select m.expires_at into v_exp from public.kiroku_memberships m where m.user_id = p_uid;
    if not found then
        return query select false, false, false, null::timestamptz;
        return;
    end if;
    return query select
        true,
        v_exp is not null and v_exp > now(),
        v_exp is not null and v_exp + interval '90 days' > now(),
        v_exp;
end;
$$;

-- 服务端校验个人记录载荷（口径与本地 validate.rs 一致：十分位整数）
create or replace function public.kiroku_validate_record(p_record jsonb)
returns void
language plpgsql immutable security definer set search_path = '' as $$
declare v_dim text; v_val jsonb; v_num numeric;
begin
    if p_record is null or jsonb_typeof(p_record) <> 'object' then
        raise invalid_parameter_value using message = 'RECORD_NOT_OBJECT';
    end if;
    if (p_record ? 'score') and jsonb_typeof(p_record->'score') is distinct from 'null' then
        if jsonb_typeof(p_record->'score') <> 'number' then
            raise invalid_parameter_value using message = 'BAD_SCORE';
        end if;
        v_num := (p_record->>'score')::numeric;
        if v_num < 0 or v_num > 100 or v_num <> floor(v_num) then
            raise invalid_parameter_value using message = 'BAD_SCORE';
        end if;
    end if;
    if jsonb_typeof(p_record->'status') <> 'string'
       or (p_record->>'status') not in ('completed', 'watching', 'planned') then
        raise invalid_parameter_value using message = 'BAD_STATUS';
    end if;
    if (p_record ? 'progress') and jsonb_typeof(p_record->'progress') is distinct from 'null' then
        if jsonb_typeof(p_record->'progress') <> 'number' then
            raise invalid_parameter_value using message = 'BAD_PROGRESS';
        end if;
        v_num := (p_record->>'progress')::numeric;
        if v_num < 0 or v_num <> floor(v_num) then
            raise invalid_parameter_value using message = 'BAD_PROGRESS';
        end if;
    end if;
    if (p_record ? 'review') and jsonb_typeof(p_record->'review') is distinct from 'null' then
        if jsonb_typeof(p_record->'review') <> 'string' or length(p_record->>'review') > 5000 then
            raise invalid_parameter_value using message = 'BAD_REVIEW';
        end if;
    end if;
    if (p_record ? 'tier_sync_id') and jsonb_typeof(p_record->'tier_sync_id') is distinct from 'null'
       and jsonb_typeof(p_record->'tier_sync_id') <> 'string' then
        raise invalid_parameter_value using message = 'BAD_TIER_REF';
    end if;
    if (p_record ? 'dimensions') and jsonb_typeof(p_record->'dimensions') is distinct from 'null' then
        if jsonb_typeof(p_record->'dimensions') <> 'object' then
            raise invalid_parameter_value using message = 'BAD_DIMENSIONS';
        end if;
        for v_dim, v_val in select key, value from jsonb_each(p_record->'dimensions') loop
            if v_dim not in ('story', 'characters', 'direction', 'animation', 'music') then
                raise invalid_parameter_value using message = 'BAD_DIMENSION_KEY';
            end if;
            if jsonb_typeof(v_val) is distinct from 'null' then
                if jsonb_typeof(v_val) <> 'number' then
                    raise invalid_parameter_value using message = 'BAD_DIMENSION_VALUE';
                end if;
                v_num := (v_val#>>'{}')::numeric;
                if v_num < 5 or v_num > 50 or mod(v_num, 5) <> 0 or v_num <> floor(v_num) then
                    raise invalid_parameter_value using message = 'BAD_DIMENSION_VALUE';
                end if;
            end if;
        end loop;
    end if;
end;
$$;

-- ---------- 客户端 RPC ----------

-- 权益查询：登录即可
create or replace function public.kiroku_get_entitlement()
returns jsonb
language plpgsql volatile security definer set search_path = '' as $$
declare
    v_uid uuid;
    v_state public.kiroku_sync_state;
    v_mem record;
begin
    v_uid := public.kiroku_require_user();
    v_state := public.kiroku_ensure_state(v_uid);
    select * into v_mem from public.kiroku_membership_status(v_uid);
    return jsonb_build_object(
        'user_id', v_uid,
        'epoch', v_state.epoch,
        'expires_at', v_mem.expires_at,
        'member_active', v_mem.active,
        'in_retention', v_mem.in_retention,
        'retention_until', case when v_mem.expires_at is null then null
                                else v_mem.expires_at + interval '90 days' end
    );
end;
$$;

-- 兑换码核销：同 request_id 幂等重放；码行锁保证一码一主；membership 行锁保证多码并发累加
create or replace function public.kiroku_redeem_code(p_code text, p_request_id text)
returns jsonb
language plpgsql volatile security definer set search_path = '' as $$
declare
    v_uid uuid;
    v_norm text;
    v_hash text;
    v_code public.kiroku_redemption_codes;
    v_mem_exp timestamptz;
    v_event_id bigint;
    v_prior record;
begin
    v_uid := public.kiroku_require_user();
    if p_request_id is null or length(p_request_id) = 0 or length(p_request_id) > 80 then
        raise invalid_parameter_value using message = 'BAD_REQUEST_ID';
    end if;
    v_norm := upper(btrim(p_code));
    if length(v_norm) < 8 or length(v_norm) > 64 then
        raise invalid_parameter_value using message = 'BAD_CODE_FORMAT';
    end if;
    v_hash := encode(extensions.digest(v_norm, 'sha256'), 'hex');

    -- 同账号同 request_id 的并发/重试串行化：后到的请求等前者提交后直接读到已存结果，
    -- 避免两个请求各自通过 prior-check、双双发券（unique 冲突只能撤销记录、撤销不了已加的时长）
    perform pg_advisory_xact_lock(hashtextextended(v_uid::text || '/' || p_request_id, 0));

    -- 同账号同 request_id 重试 → 直接返回原结果
    select r.code_hash, r.redeemed_at, e.days_delta, m.expires_at
      into v_prior
      from public.kiroku_redemptions r
      join public.kiroku_entitlement_events e on e.id = r.event_id
      left join public.kiroku_memberships m on m.user_id = r.user_id
     where r.user_id = v_uid and r.request_id = p_request_id;
    if found then
        return jsonb_build_object(
            'status', 'replayed',
            'expires_at', v_prior.expires_at,
            'days_added', v_prior.days_delta
        );
    end if;

    select * into v_code from public.kiroku_redemption_codes
     where code_hash = v_hash for update;
    if not found then
        return jsonb_build_object('status', 'invalid_code');
    end if;
    if v_code.status = 'disabled' then
        return jsonb_build_object('status', 'disabled');
    end if;
    if v_code.status = 'redeemed' then
        return jsonb_build_object('status', 'already_redeemed');
    end if;
    if v_code.redeem_by is not null and v_code.redeem_by < now() then
        return jsonb_build_object('status', 'expired');
    end if;

    -- 锁定本账号权益行（首建也行）
    insert into public.kiroku_memberships (user_id) values (v_uid)
    on conflict (user_id) do nothing;
    select expires_at into v_mem_exp from public.kiroku_memberships
     where user_id = v_uid for update;

    v_mem_exp := greatest(coalesce(v_mem_exp, '-infinity'::timestamptz), now())
                 + v_code.duration_days * interval '1 day';

    update public.kiroku_memberships set expires_at = v_mem_exp, updated_at = now()
     where user_id = v_uid;
    update public.kiroku_redemption_codes
       set status = 'redeemed', redeemed_by = v_uid, redeemed_at = now()
     where code_hash = v_hash;

    insert into public.kiroku_entitlement_events
        (user_id, kind, days_delta, source_kind, source_ref)
    values (v_uid, 'redeem', v_code.duration_days, 'redemption_code', v_hash)
    returning id into v_event_id;

    begin
        insert into public.kiroku_redemptions
            (code_hash, user_id, request_id, event_id)
        values (v_hash, v_uid, p_request_id, v_event_id);
    exception when unique_violation then
        -- 同 request_id 并发：本次撤销，返回已存在结果
        raise notice 'concurrent request_id, reading prior result';
        select r.code_hash, r.redeemed_at, e.days_delta, m.expires_at
          into v_prior
          from public.kiroku_redemptions r
          join public.kiroku_entitlement_events e on e.id = r.event_id
          left join public.kiroku_memberships m on m.user_id = r.user_id
         where r.user_id = v_uid and r.request_id = p_request_id;
        return jsonb_build_object(
            'status', 'replayed',
            'expires_at', v_prior.expires_at,
            'days_added', v_prior.days_delta
        );
    end;

    return jsonb_build_object(
        'status', 'redeemed',
        'expires_at', v_mem_exp,
        'days_added', v_code.duration_days
    );
end;
$$;

-- 单条同步操作应用器：返回 {status: applied|conflict, server_version, remote?, change_payload?}
create or replace function public.kiroku_apply_op(p_uid uuid, p_op jsonb)
returns jsonb
language plpgsql volatile security definer set search_path = '' as $$
declare
    v_type text := p_op->>'entity_type';
    v_key text := p_op->>'entity_key';
    v_op text := p_op->>'op_type';
    v_base bigint := coalesce((p_op->>'base_server_version')::bigint, 0);
    v_payload jsonb := p_op->'payload';
    v_subject bigint;
    v_row record;
    v_new bigint;
    v_found boolean;
begin
    if v_type = 'record' then
        if v_key !~ '^[0-9]+$' then
            raise invalid_parameter_value using message = 'BAD_ENTITY_KEY';
        end if;
        v_subject := v_key::bigint;
        select * into v_row from public.kiroku_library_entries
         where user_id = p_uid and subject_id = v_subject;
        v_found := found;   -- perform 会重置 FOUND，必须先落变量
        if v_op = 'upsert' then
            perform public.kiroku_validate_record(v_payload->'record');
            if octet_length(coalesce(v_payload->>'snapshot', '')) > 32768 then
                raise invalid_parameter_value using message = 'SNAPSHOT_TOO_LARGE';
            end if;
            if not v_found then
                if v_base > 0 then
                    return jsonb_build_object('status', 'conflict', 'reason', 'missing_base');
                end if;
                insert into public.kiroku_library_entries
                    (user_id, subject_id, record, snapshot, server_version)
                values (p_uid, v_subject, v_payload->'record', v_payload->'snapshot', 1);
                v_new := 1;
            else
                if v_row.deleted_at is not null or v_row.server_version <> v_base then
                    return jsonb_build_object(
                        'status', 'conflict', 'reason', 'version_mismatch',
                        'server_version', v_row.server_version,
                        'remote', jsonb_build_object('record', v_row.record, 'snapshot', v_row.snapshot,
                                                     'deleted', v_row.deleted_at is not null));
                end if;
                v_new := v_row.server_version + 1;
                update public.kiroku_library_entries
                   set record = v_payload->'record', snapshot = v_payload->'snapshot',
                       server_version = v_new, updated_at = now()
                 where user_id = p_uid and subject_id = v_subject;
            end if;
            return jsonb_build_object('status', 'applied', 'server_version', v_new,
                                      'change_payload', v_payload);
        elsif v_op = 'delete' then
            if not v_found or v_row.deleted_at is not null then
                return jsonb_build_object('status', 'applied', 'server_version', v_base,
                                          'change_payload', null);
            end if;
            if v_row.server_version <> v_base then
                return jsonb_build_object(
                    'status', 'conflict', 'reason', 'version_mismatch',
                    'server_version', v_row.server_version,
                    'remote', jsonb_build_object('record', v_row.record, 'snapshot', v_row.snapshot,
                                                 'deleted', false));
            end if;
            v_new := v_row.server_version + 1;
            update public.kiroku_library_entries
               set record = null, snapshot = null, deleted_at = now(),
                   server_version = v_new, updated_at = now()
             where user_id = p_uid and subject_id = v_subject;
            return jsonb_build_object('status', 'applied', 'server_version', v_new,
                                      'change_payload', null);
        else
            raise invalid_parameter_value using message = 'BAD_OP_TYPE';
        end if;
    elsif v_type = 'tier' then
        select * into v_row from public.kiroku_tiers
         where user_id = p_uid and sync_id = v_key;
        v_found := found;
        if v_op = 'upsert' then
            if v_payload is null or jsonb_typeof(v_payload->'name') <> 'string'
               or length(v_payload->>'name') = 0 or length(v_payload->>'name') > 64 then
                raise invalid_parameter_value using message = 'BAD_TIER_NAME';
            end if;
            if not v_found then
                if v_base > 0 then
                    return jsonb_build_object('status', 'conflict', 'reason', 'missing_base');
                end if;
                insert into public.kiroku_tiers
                    (user_id, sync_id, name, description, color, builtin, server_version)
                values (p_uid, v_key, v_payload->>'name',
                        coalesce(v_payload->>'description', ''),
                        coalesce(v_payload->>'color', ''),
                        coalesce((v_payload->>'builtin')::boolean, false), 1);
                v_new := 1;
            else
                if v_row.deleted_at is not null or v_row.server_version <> v_base then
                    return jsonb_build_object(
                        'status', 'conflict', 'reason', 'version_mismatch',
                        'server_version', v_row.server_version,
                        'remote', to_jsonb(v_row));
                end if;
                v_new := v_row.server_version + 1;
                update public.kiroku_tiers
                   set name = v_payload->>'name',
                       description = coalesce(v_payload->>'description', ''),
                       color = coalesce(v_payload->>'color', ''),
                       server_version = v_new
                 where user_id = p_uid and sync_id = v_key;
            end if;
            return jsonb_build_object('status', 'applied', 'server_version', v_new,
                                      'change_payload', v_payload);
        elsif v_op = 'delete' then
            if not v_found or v_row.deleted_at is not null then
                return jsonb_build_object('status', 'applied', 'server_version', v_base,
                                          'change_payload', null);
            end if;
            if v_row.server_version <> v_base then
                return jsonb_build_object(
                    'status', 'conflict', 'reason', 'version_mismatch',
                    'server_version', v_row.server_version,
                    'remote', to_jsonb(v_row));
            end if;
            v_new := v_row.server_version + 1;
            update public.kiroku_tiers set deleted_at = now(), server_version = v_new
             where user_id = p_uid and sync_id = v_key;
            return jsonb_build_object('status', 'applied', 'server_version', v_new,
                                      'change_payload', null);
        else
            raise invalid_parameter_value using message = 'BAD_OP_TYPE';
        end if;
    elsif v_type = 'tier_order' then
        if v_op <> 'set' then
            raise invalid_parameter_value using message = 'BAD_OP_TYPE';
        end if;
        if jsonb_typeof(v_payload->'ordered_keys') <> 'array' then
            raise invalid_parameter_value using message = 'BAD_ORDER';
        end if;
        select * into v_row from public.kiroku_tier_order where user_id = p_uid;
        if not found then   -- 紧接 select，无 perform 介入
            insert into public.kiroku_tier_order (user_id, ordered_keys, server_version)
            values (p_uid, v_payload->'ordered_keys', 1);
            v_new := 1;
        else
            if v_row.server_version <> v_base then
                return jsonb_build_object(
                    'status', 'conflict', 'reason', 'version_mismatch',
                    'server_version', v_row.server_version,
                    'remote', jsonb_build_object('ordered_keys', v_row.ordered_keys));
            end if;
            v_new := v_row.server_version + 1;
            update public.kiroku_tier_order
               set ordered_keys = v_payload->'ordered_keys', server_version = v_new
             where user_id = p_uid;
        end if;
        return jsonb_build_object('status', 'applied', 'server_version', v_new,
                                  'change_payload', v_payload);
    else
        raise invalid_parameter_value using message = 'BAD_ENTITY_TYPE';
    end if;
end;
$$;

-- 推送：权益实时检查 → 逐 op 幂等/版本校验 → 同事务分配 seq 写日志；遇冲突停批
create or replace function public.kiroku_sync_push(p_ops jsonb)
returns jsonb
language plpgsql volatile security definer set search_path = '' as $$
declare
    v_uid uuid;
    v_state public.kiroku_sync_state;
    v_mem record;
    v_op jsonb;
    v_res jsonb;
    v_prev jsonb;
    v_results jsonb := '[]'::jsonb;
    v_op_id text;
    v_seq bigint;
begin
    v_uid := public.kiroku_require_user();
    if p_ops is null or jsonb_typeof(p_ops) <> 'array' then
        raise invalid_parameter_value using message = 'OPS_NOT_ARRAY';
    end if;
    if jsonb_array_length(p_ops) > 50 then
        raise invalid_parameter_value using message = 'BATCH_TOO_LARGE';
    end if;

    select * into v_mem from public.kiroku_membership_status(v_uid);
    v_state := public.kiroku_ensure_state(v_uid);   -- 行锁，串行化推送

    if not v_mem.active then
        return jsonb_build_object(
            'epoch', v_state.epoch,
            'status', 'entitlement_expired',
            'results', '[]'::jsonb);
    end if;

    for v_op in select value from jsonb_array_elements(p_ops) loop
        v_op_id := v_op->>'op_id';
        if v_op_id is null or length(v_op_id) = 0 or length(v_op_id) > 80
           or jsonb_typeof(v_op) <> 'object' then
            raise invalid_parameter_value using message = 'BAD_OP_ENVELOPE';
        end if;

        select result into v_prev from public.kiroku_sync_operations
         where user_id = v_uid and op_id = v_op_id;
        if found then
            v_results := v_results || jsonb_build_object(
                'op_id', v_op_id, 'status', 'replayed',
                'server_version', v_prev->'server_version');
            continue;
        end if;

        v_res := public.kiroku_apply_op(v_uid, v_op);

        if v_res->>'status' = 'applied' then
            update public.kiroku_sync_state set seq = seq + 1
             where user_id = v_uid returning seq into v_seq;
            insert into public.kiroku_sync_changes
                (user_id, seq, entity_type, entity_key, op, payload, server_version, op_id)
            values (v_uid, v_seq, v_op->>'entity_type', v_op->>'entity_key',
                    v_op->>'op_type', v_res->'change_payload',
                    (v_res->>'server_version')::bigint, v_op_id);
            insert into public.kiroku_sync_operations
                (user_id, op_id, entity_type, entity_key, result)
            values (v_uid, v_op_id, v_op->>'entity_type', v_op->>'entity_key',
                    v_res - 'change_payload');
        else
            -- 冲突不写入幂等表：重试按最新状态重新评估；停批保持顺序语义
            v_results := v_results || (jsonb_build_object('op_id', v_op_id) || v_res);
            exit;
        end if;
        v_results := v_results || (jsonb_build_object('op_id', v_op_id) || (v_res - 'change_payload'));
    end loop;

    return jsonb_build_object(
        'epoch', v_state.epoch,
        'status', 'ok',
        'results', v_results);
end;
$$;

-- 拉取：cursor=0 返回一致快照（与边界同事务读取）；否则按 seq 增量分页
create or replace function public.kiroku_sync_pull(p_cursor bigint, p_limit integer default 500)
returns jsonb
language plpgsql stable security definer set search_path = '' as $$
declare
    v_uid uuid;
    v_state public.kiroku_sync_state;
    v_mem record;
    v_lim int := least(greatest(coalesce(p_limit, 500), 1), 500);
    v_items jsonb;
    v_changes jsonb;
    v_next bigint;
    v_more boolean;
    v_min_seq bigint;
begin
    v_uid := public.kiroku_require_user();
    select * into v_mem from public.kiroku_membership_status(v_uid);
    if not v_mem.active and not v_mem.in_retention then
        return jsonb_build_object('status', 'no_access');
    end if;

    select * into v_state from public.kiroku_sync_state where user_id = v_uid;
    if not found then
        return jsonb_build_object('epoch', 1, 'mode', 'snapshot',
                                  'boundary', 0, 'items', '[]'::jsonb,
                                  'next_cursor', 0, 'has_more', false);
    end if;

    if coalesce(p_cursor, 0) = 0 then
        -- 同事务快照：boundary 与 items 天然一致
        select jsonb_agg(item order by ord) into v_items from (
            select 1 as ord, jsonb_build_object(
                       'entity_type', 'tier', 'entity_key', t.sync_id, 'op', 'upsert',
                       'server_version', t.server_version,
                       'payload', jsonb_build_object('name', t.name, 'description', t.description,
                                                     'color', t.color, 'builtin', t.builtin)) as item
              from public.kiroku_tiers t
             where t.user_id = v_uid and t.deleted_at is null
            union all
            select 2, jsonb_build_object(
                       'entity_type', 'tier_order', 'entity_key', 'tier_order', 'op', 'set',
                       'server_version', o.server_version,
                       'payload', jsonb_build_object('ordered_keys', o.ordered_keys))
              from public.kiroku_tier_order o
             where o.user_id = v_uid
            union all
            select 3, jsonb_build_object(
                       'entity_type', 'record', 'entity_key', e.subject_id::text, 'op', 'upsert',
                       'server_version', e.server_version,
                       'payload', jsonb_build_object('record', e.record, 'snapshot', e.snapshot))
              from public.kiroku_library_entries e
             where e.user_id = v_uid and e.deleted_at is null
        ) s;
        return jsonb_build_object(
            'epoch', v_state.epoch, 'mode', 'snapshot',
            'boundary', v_state.seq,
            'items', coalesce(v_items, '[]'::jsonb),
            'next_cursor', v_state.seq, 'has_more', false);
    end if;

    -- 游标过期：最旧保留 seq 大于 cursor+1 说明中间段已被清理
    select min(seq) into v_min_seq from public.kiroku_sync_changes where user_id = v_uid;
    if v_min_seq is not null and v_min_seq > p_cursor + 1 then
        return jsonb_build_object('epoch', v_state.epoch, 'status', 'cursor_expired');
    end if;

    select jsonb_agg(to_jsonb(c)), max(c.seq)
      into v_changes, v_next
      from (
        select seq, entity_type, entity_key, op, payload, server_version, committed_at
          from public.kiroku_sync_changes
         where user_id = v_uid and seq > p_cursor
         order by seq limit v_lim + 1
      ) c;
    v_more := jsonb_array_length(coalesce(v_changes, '[]'::jsonb)) > v_lim;
    if v_more then
        v_changes := (select jsonb_agg(elem) from (
            select elem from jsonb_array_elements(v_changes) elem
            order by (elem->>'seq')::bigint limit v_lim) t);
        v_next := (select max((elem->>'seq')::bigint)
                     from jsonb_array_elements(v_changes) elem);
    end if;

    return jsonb_build_object(
        'epoch', v_state.epoch, 'mode', 'changes',
        'changes', coalesce(v_changes, '[]'::jsonb),
        'next_cursor', coalesce(v_next, p_cursor),
        'has_more', v_more);
end;
$$;

-- 清空云端库：个人数据删除但保留会员/兑换记录；epoch+1 使旧设备游标失效
create or replace function public.kiroku_delete_cloud_library(p_confirm text)
returns jsonb
language plpgsql volatile security definer set search_path = '' as $$
declare
    v_uid uuid;
    v_state public.kiroku_sync_state;
begin
    v_uid := public.kiroku_require_user();
    if p_confirm is distinct from 'DELETE' then
        raise invalid_parameter_value using message = 'CONFIRM_REQUIRED';
    end if;
    v_state := public.kiroku_ensure_state(v_uid);
    delete from public.kiroku_library_entries where user_id = v_uid;
    delete from public.kiroku_tiers where user_id = v_uid;
    delete from public.kiroku_tier_order where user_id = v_uid;
    delete from public.kiroku_sync_changes where user_id = v_uid;
    delete from public.kiroku_sync_operations where user_id = v_uid;
    update public.kiroku_sync_state set epoch = epoch + 1, seq = 0
     where user_id = v_uid
    returning epoch into v_state.epoch;
    return jsonb_build_object('status', 'deleted', 'epoch', v_state.epoch);
end;
$$;

-- ---------- 管理 RPC（仅 service_role） ----------

-- 批量生成兑换码：返回明文码（仅此一次可见），库中只存 sha256 摘要
create or replace function public.kiroku_admin_gen_codes(
    p_batch text, p_days integer, p_count integer, p_redeem_by timestamptz default null
)
returns setof text
language plpgsql volatile security definer set search_path = '' as $$
declare
    i int := 0;
    v_code text;
begin
    if auth.role() is distinct from 'service_role' then
        raise insufficient_privilege using message = 'ADMIN_ONLY';
    end if;
    if p_days is null or p_days <= 0 or p_count is null or p_count < 1 or p_count > 1000 then
        raise invalid_parameter_value using message = 'BAD_GEN_PARAMS';
    end if;
    while i < p_count loop
        v_code := 'KIR-'
                  || upper(substr(encode(extensions.gen_random_bytes(8), 'hex'), 1, 4))
                  || '-' || upper(substr(encode(extensions.gen_random_bytes(8), 'hex'), 5, 4))
                  || '-' || upper(substr(encode(extensions.gen_random_bytes(8), 'hex'), 9, 4))
                  || '-' || upper(substr(encode(extensions.gen_random_bytes(8), 'hex'), 13, 4));
        begin
            insert into public.kiroku_redemption_codes
                (code_hash, duration_days, batch_id, redeem_by)
            values (encode(extensions.digest(v_code, 'sha256'), 'hex'),
                    p_days, p_batch, p_redeem_by);
            return next v_code;
            i := i + 1;
        exception when unique_violation then
            continue;  -- 撞哈希重抽
        end;
    end loop;
    return;
end;
$$;
