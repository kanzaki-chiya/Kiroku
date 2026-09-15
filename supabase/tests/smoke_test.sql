-- Kiroku 云同步 · 本地冒烟测试
-- 用法：先起带 auth 桩的 Postgres（见文件尾部注释），依次跑 migrations/ 三个文件，再跑本文件。
-- 依赖会话变量模拟登录：set kiroku.test_uid / kiroku.test_role。
\set ON_ERROR_STOP on
\set QUIET on
\pset format unaligned
\pset tuples_only on

create or replace function pg_temp.assert_true(p boolean, p_msg text) returns void language plpgsql as $$
begin if not coalesce(p,false) then raise exception 'FAIL: %', p_msg; else raise notice 'PASS: %', p_msg; end if; end $$;

-- 1. 未登录拒绝
set role authenticated; set kiroku.test_uid=''; set kiroku.test_role='authenticated';
do $$ begin perform public.kiroku_get_entitlement(); raise exception 'FAIL: anon allowed';
exception when insufficient_privilege then raise notice 'PASS: anon rejected'; end $$;

-- 2. 登录用户、无会员
set kiroku.test_uid='11111111-1111-1111-1111-111111111111';
do $$ declare r jsonb; begin
  select public.kiroku_get_entitlement() into r;
  perform pg_temp.assert_true(r->>'member_active'='false', 'new user not member');
  perform pg_temp.assert_true((r->>'epoch')='1', 'epoch=1');
end $$;

-- 3. 表级权限被拒
do $$ begin perform 1 from public.kiroku_memberships limit 1; raise exception 'FAIL: table readable';
exception when insufficient_privilege then raise notice 'PASS: direct table access denied'; end $$;
do $$ begin perform public.kiroku_admin_gen_codes('x',30,1,null); raise exception 'FAIL: admin fn open';
exception when insufficient_privilege then raise notice 'PASS: admin fn denied for authenticated'; end $$;

-- 4. 生成兑换码（service_role）
reset role; set kiroku.test_role='service_role'; set kiroku.test_uid='';
create temp table codes(c text);
insert into codes select public.kiroku_admin_gen_codes('beta1', 30, 2, null);
grant select on codes to authenticated;
do $$ begin perform pg_temp.assert_true((select count(*) from codes)=2, 'gen 2 codes'); end $$;
do $$ begin perform pg_temp.assert_true(
  (select count(*) from public.kiroku_redemption_codes where code_hash like '%KIR%')=0,
  'only hash stored'); end $$;

-- 5. u2 无会员：拉取/推送被拒
set role authenticated; set kiroku.test_role='authenticated';
set kiroku.test_uid='22222222-2222-2222-2222-222222222222';
do $$ declare r jsonb; begin
  select public.kiroku_sync_pull(0, 500) into r;
  perform pg_temp.assert_true(r->>'status'='no_access', 'non-member pull no_access');
  select public.kiroku_sync_push('[{"op_id":"o1","entity_type":"record","entity_key":"1","op_type":"upsert","payload":{"record":{"status":"watching"}},"base_server_version":0}]'::jsonb) into r;
  perform pg_temp.assert_true(r->>'status'='entitlement_expired', 'non-member push expired');
end $$;

-- 6. u1 兑换：幂等、重复码、无效码
set kiroku.test_uid='11111111-1111-1111-1111-111111111111';
do $$ declare r jsonb; r2 jsonb; c1 text; begin
  select c into c1 from codes order by c limit 1;
  select public.kiroku_redeem_code(c1, 'req-1') into r;
  perform pg_temp.assert_true(r->>'status'='redeemed', 'redeem ok');
  perform pg_temp.assert_true((r->>'days_added')='30', 'days=30');
  select public.kiroku_redeem_code(c1, 'req-1') into r2;
  perform pg_temp.assert_true(r2->>'status'='replayed' and r2->>'expires_at'=r->>'expires_at', 'same request_id replays');
  select public.kiroku_redeem_code(c1, 'req-2') into r2;
  perform pg_temp.assert_true(r2->>'status'='already_redeemed', 'code single-use');
  select public.kiroku_redeem_code('KIR-NOPE-NOPE-NOPE-NOPE', 'req-3') into r2;
  perform pg_temp.assert_true(r2->>'status'='invalid_code', 'invalid code');
end $$;

-- u2 用同一码被拒；用 code2 成功
set kiroku.test_uid='22222222-2222-2222-2222-222222222222';
do $$ declare r jsonb; c1 text; c2 text; begin
  select c into c1 from codes order by c limit 1;
  select c into c2 from codes order by c offset 1 limit 1;
  select public.kiroku_redeem_code(c1, 'req-9') into r;
  perform pg_temp.assert_true(r->>'status'='already_redeemed', 'cross-user same code rejected');
  select public.kiroku_redeem_code(c2, 'req-10') into r;
  perform pg_temp.assert_true(r->>'status'='redeemed', 'u2 own code ok');
end $$;

-- 7. u1 推送/幂等/冲突停批
set kiroku.test_uid='11111111-1111-1111-1111-111111111111';
do $$ declare r jsonb; elem jsonb; begin
  select public.kiroku_sync_push(jsonb_build_array(
    jsonb_build_object('op_id','op-add','entity_type','tier','entity_key','builtin:S','op_type','upsert',
      'payload',jsonb_build_object('name','S','description','私心珍藏','color','#bd592e','builtin',true),'base_server_version',0),
    jsonb_build_object('op_id','op-rec','entity_type','record','entity_key','400602','op_type','upsert',
      'payload',jsonb_build_object('record',jsonb_build_object('score',85,'status','watching','progress',5,'review','好看','tier_sync_id','builtin:S','dimensions',jsonb_build_object('story',45)),
                                   'snapshot',jsonb_build_object('name','测试番','cover_url','x')),
      'base_server_version',0))) into r;
  perform pg_temp.assert_true(r->>'status'='ok', 'push ok');
  perform pg_temp.assert_true(jsonb_array_length(r->'results')=2, 'two results');
  elem := r->'results'->1;
  perform pg_temp.assert_true(elem->>'status'='applied' and (elem->>'server_version')='1', 'record applied v1');

  select public.kiroku_sync_push(jsonb_build_array(
    jsonb_build_object('op_id','op-rec','entity_type','record','entity_key','400602','op_type','upsert',
      'payload',jsonb_build_object('record',jsonb_build_object('score',99,'status','completed'),'snapshot',null),
      'base_server_version',1))) into r;
  elem := r->'results'->0;
  perform pg_temp.assert_true(elem->>'status'='replayed', 'op_id replay');
  select public.kiroku_sync_pull(0, 500) into r;
  perform pg_temp.assert_true(
    (select it->'payload'->'record'->>'score' from jsonb_array_elements(r->'items') it
      where it->>'entity_type'='record' and it->>'entity_key'='400602')='85',
    'replay did not overwrite');

  select public.kiroku_sync_push(jsonb_build_array(
    jsonb_build_object('op_id','op-stale','entity_type','record','entity_key','400602','op_type','upsert',
      'payload',jsonb_build_object('record',jsonb_build_object('status','completed'),'snapshot',null),
      'base_server_version',0),
    jsonb_build_object('op_id','op-after','entity_type','record','entity_key','7','op_type','upsert',
      'payload',jsonb_build_object('record',jsonb_build_object('status','planned'),'snapshot',null),
      'base_server_version',0))) into r;
  elem := r->'results'->0;
  perform pg_temp.assert_true(elem->>'status'='conflict' and (elem->>'server_version')='1', 'stale base conflicts');
  perform pg_temp.assert_true(jsonb_array_length(r->'results')=1, 'batch stops at conflict');
end $$;

-- 8. 跨账号隔离：u2 拉不到 u1 数据
set kiroku.test_uid='22222222-2222-2222-2222-222222222222';
do $$ declare r jsonb; begin
  select public.kiroku_sync_pull(0, 500) into r;
  perform pg_temp.assert_true(jsonb_array_length(coalesce(r->'items','[]'::jsonb))=0, 'u2 snapshot empty');
end $$;

-- 9. u1 拉取：快照 + 增量 + 删除传播
set kiroku.test_uid='11111111-1111-1111-1111-111111111111';
do $$ declare r jsonb; boundary bigint; ch jsonb; begin
  select public.kiroku_sync_pull(0, 500) into r;
  boundary := (r->>'boundary')::bigint;
  perform pg_temp.assert_true(jsonb_array_length(r->'items')=2, 'snapshot has tier+record');
  perform pg_temp.assert_true(boundary=2, 'boundary=2');

  perform public.kiroku_sync_push(jsonb_build_array(
    jsonb_build_object('op_id','op-del','entity_type','record','entity_key','400602','op_type','delete',
      'payload',null,'base_server_version',1)));
  select public.kiroku_sync_pull(boundary, 500) into r;
  ch := r->'changes'->0;
  perform pg_temp.assert_true(ch->>'op'='delete' and ch->>'entity_key'='400602', 'delete propagates');

  select public.kiroku_sync_pull(0, 500) into r;
  perform pg_temp.assert_true(jsonb_array_length(r->'items')=1, 'snapshot excludes tombstone');
end $$;

-- 10. 清空云端库 → epoch+1
do $$ declare r jsonb; begin
  select public.kiroku_delete_cloud_library('DELETE') into r;
  perform pg_temp.assert_true((r->>'epoch')='2', 'epoch bumped');
  select public.kiroku_sync_pull(0, 500) into r;
  perform pg_temp.assert_true(jsonb_array_length(r->'items')=0, 'cloud library empty');
end $$;

do $$ begin raise notice '==== ALL TESTS DONE ===='; end $$;

-- 本地跑法：
--   1) initdb + pg_ctl 起临时实例
--   2) 建 auth 桩：schema auth + auth.users + auth.uid()/auth.role() 读 kiroku.test_uid/test_role
--      + schema extensions + pgcrypto + 角色 anon/authenticated/service_role
--   3) psql -f migrations/202609160001..3.sql -f tests/smoke_test.sql
