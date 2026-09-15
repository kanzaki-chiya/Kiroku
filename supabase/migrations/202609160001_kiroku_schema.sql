-- Kiroku 云同步 · Migration 001：schema
-- 对应 SYNC_DESIGN.md §4.1。所有表带 kiroku_ 前缀，只增不改。
-- 客户端不直接访问任何表；访问面仅由 SECURITY DEFINER 函数提供。

create extension if not exists pgcrypto with schema extensions;

-- 每用户同步状态：变更序号计数 + 云端库代次
create table public.kiroku_sync_state (
    user_id uuid primary key references auth.users (id) on delete cascade,
    epoch bigint not null default 1,
    seq bigint not null default 0
);

-- 收藏记录：同步单元。record 为个人记录载荷，snapshot 为 Bangumi 资料快照
create table public.kiroku_library_entries (
    user_id uuid not null references auth.users (id) on delete cascade,
    subject_id bigint not null,
    record jsonb,
    snapshot jsonb,
    server_version bigint not null default 1,
    deleted_at timestamptz,
    updated_at timestamptz not null default now(),
    primary key (user_id, subject_id)
);

-- 自定义/内置分档：以 sync_id 为稳定标识，名称可改
create table public.kiroku_tiers (
    user_id uuid not null references auth.users (id) on delete cascade,
    sync_id text not null,
    name text not null,
    description text not null default '',
    color text not null default '',
    builtin boolean not null default false,
    server_version bigint not null default 1,
    deleted_at timestamptz,
    primary key (user_id, sync_id)
);

-- 分档排序：每用户单例实体，整体有序列表一个版本
create table public.kiroku_tier_order (
    user_id uuid primary key references auth.users (id) on delete cascade,
    ordered_keys jsonb not null default '[]'::jsonb,
    server_version bigint not null default 1
);

-- 变更日志：seq 由 kiroku_sync_state.seq 在行锁内 +1 分配，同用户严格有序无空洞
create table public.kiroku_sync_changes (
    user_id uuid not null references auth.users (id) on delete cascade,
    seq bigint not null,
    entity_type text not null check (entity_type in ('record', 'tier', 'tier_order')),
    entity_key text not null,
    op text not null check (op in ('upsert', 'delete', 'set')),
    payload jsonb,
    server_version bigint not null,
    op_id text,
    committed_at timestamptz not null default now(),
    primary key (user_id, seq)
);

-- 操作幂等记录：op_id 重试返回已记录结果，不重复应用
create table public.kiroku_sync_operations (
    user_id uuid not null references auth.users (id) on delete cascade,
    op_id text not null,
    entity_type text not null,
    entity_key text not null,
    result jsonb not null,
    applied_at timestamptz not null default now(),
    primary key (user_id, op_id)
);

-- 会员：每个账号当前同步服务到期时间
create table public.kiroku_memberships (
    user_id uuid primary key references auth.users (id) on delete cascade,
    expires_at timestamptz,
    updated_at timestamptz not null default now()
);

-- 权益流水：兑换/购买/补偿/撤销均留痕
create table public.kiroku_entitlement_events (
    id bigint generated always as identity primary key,
    user_id uuid not null references auth.users (id) on delete cascade,
    kind text not null check (kind in ('redeem', 'purchase', 'grant', 'revoke')),
    days_delta bigint not null,
    source_kind text,
    source_ref text,
    created_at timestamptz not null default now()
);
create index kiroku_entitlement_events_user on public.kiroku_entitlement_events (user_id, id);

-- 兑换码：只存 sha256 摘要，原码仅生成时导出
create table public.kiroku_redemption_codes (
    code_hash text primary key,
    duration_days integer not null check (duration_days > 0),
    batch_id text,
    redeem_by timestamptz,
    status text not null default 'active' check (status in ('active', 'redeemed', 'disabled')),
    redeemed_by uuid references auth.users (id),
    redeemed_at timestamptz,
    created_at timestamptz not null default now()
);
create index kiroku_redemption_codes_batch on public.kiroku_redemption_codes (batch_id);

-- 兑换记录：码一次性（PK），request_id 保证同账号重试幂等
create table public.kiroku_redemptions (
    code_hash text not null references public.kiroku_redemption_codes (code_hash),
    user_id uuid not null references auth.users (id) on delete cascade,
    request_id text not null,
    event_id bigint references public.kiroku_entitlement_events (id),
    redeemed_at timestamptz not null default now(),
    primary key (code_hash),
    unique (user_id, request_id)
);
create index kiroku_redemptions_user on public.kiroku_redemptions (user_id, redeemed_at);
