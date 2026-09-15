-- Kiroku 云同步 · Migration 003：RLS + GRANT 收紧
-- 原则：anon/authenticated 对所有业务表零权限；客户端只能 EXECUTE 受控函数。
-- RLS 作为纵深防御兜底：即使授权失误也不放大暴露面。

-- ---------- 表级 RLS ----------

alter table public.kiroku_sync_state enable row level security;
alter table public.kiroku_library_entries enable row level security;
alter table public.kiroku_tiers enable row level security;
alter table public.kiroku_tier_order enable row level security;
alter table public.kiroku_sync_changes enable row level security;
alter table public.kiroku_sync_operations enable row level security;
alter table public.kiroku_memberships enable row level security;
alter table public.kiroku_entitlement_events enable row level security;
alter table public.kiroku_redemption_codes enable row level security;
alter table public.kiroku_redemptions enable row level security;

-- 账号隔离策略（防御纵深；正常路径下这些表无 GRANT，策略不会被触发）
create policy kiroku_isolation on public.kiroku_sync_state
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_library_entries
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_tiers
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_tier_order
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_sync_changes
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_sync_operations
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_memberships
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_entitlement_events
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
create policy kiroku_isolation on public.kiroku_redemptions
    for all to authenticated
    using (user_id = auth.uid()) with check (user_id = auth.uid());
-- kiroku_redemption_codes：启用 RLS 且无任何策略 = 对非 owner 全拒

-- ---------- 表权限：全部收回 ----------

revoke all on public.kiroku_sync_state from public, anon, authenticated;
revoke all on public.kiroku_library_entries from public, anon, authenticated;
revoke all on public.kiroku_tiers from public, anon, authenticated;
revoke all on public.kiroku_tier_order from public, anon, authenticated;
revoke all on public.kiroku_sync_changes from public, anon, authenticated;
revoke all on public.kiroku_sync_operations from public, anon, authenticated;
revoke all on public.kiroku_memberships from public, anon, authenticated;
revoke all on public.kiroku_entitlement_events from public, anon, authenticated;
revoke all on public.kiroku_redemption_codes from public, anon, authenticated;
revoke all on public.kiroku_redemptions from public, anon, authenticated;
revoke all on sequence public.kiroku_entitlement_events_id_seq from public, anon, authenticated;

-- ---------- 函数权限：先全收，再按需授予 ----------

revoke all on function public.kiroku_require_user() from public, anon, authenticated, service_role;
revoke all on function public.kiroku_ensure_state(uuid) from public, anon, authenticated, service_role;
revoke all on function public.kiroku_membership_status(uuid) from public, anon, authenticated, service_role;
revoke all on function public.kiroku_validate_record(jsonb) from public, anon, authenticated, service_role;
revoke all on function public.kiroku_apply_op(uuid, jsonb) from public, anon, authenticated, service_role;

revoke all on function public.kiroku_get_entitlement() from public, anon, service_role;
revoke all on function public.kiroku_redeem_code(text, text) from public, anon, service_role;
revoke all on function public.kiroku_sync_push(jsonb) from public, anon, service_role;
revoke all on function public.kiroku_sync_pull(bigint, integer) from public, anon, service_role;
revoke all on function public.kiroku_delete_cloud_library(text) from public, anon, service_role;
revoke all on function public.kiroku_admin_gen_codes(text, integer, integer, timestamptz) from public, anon, authenticated;

grant execute on function public.kiroku_get_entitlement() to authenticated;
grant execute on function public.kiroku_redeem_code(text, text) to authenticated;
grant execute on function public.kiroku_sync_push(jsonb) to authenticated;
grant execute on function public.kiroku_sync_pull(bigint, integer) to authenticated;
grant execute on function public.kiroku_delete_cloud_library(text) to authenticated;

grant execute on function public.kiroku_admin_gen_codes(text, integer, integer, timestamptz) to service_role;
