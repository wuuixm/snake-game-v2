-- ==========================================
-- 多人联机贪吃蛇 — 数据库初始化脚本
-- 执行方式: psql -U pgsql -d snake_db -f db.sql
-- ==========================================

-- 1. 用户表：存储账户信息（密码经 bcrypt 哈希）
create table if not exists public.users
(
    id            serial
        primary key,
    username      varchar(50)  not null
        unique,
    password_hash varchar(255) not null,
    created_at    timestamp with time zone default current_timestamp
);

alter table public.users
    owner to pgsql;

-- 2. 排行榜表：每局结算时写入一条记录
create table if not exists public.leaderboard
(
    id        serial
        primary key,
    username  varchar(50)                        not null
        constraint fk_user
            references public.users (username)
            on delete cascade,
    score     integer                  default 0 not null,
    played_at timestamp with time zone default current_timestamp
);

alter table public.leaderboard
    owner to pgsql;

-- 3. 为 ESP32 等嵌入式设备预置硬编码账户（仅开发阶段使用）
-- INSERT INTO public.users (username, password_hash)
-- VALUES ('esp32_player', '$2b$12$...')
-- ON CONFLICT (username) DO NOTHING;

