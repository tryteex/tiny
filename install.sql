CREATE TABLE "public"."access" (
  "access_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "role_id" int8 NOT NULL,
  "access" bool NOT NULL,
  "controller_id" int8 NOT NULL
);
CREATE TABLE "public"."controller" (
  "controller_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "module" text NOT NULL,
  "class" text NOT NULL,
  "action" text NOT NULL,
  "desc" jsonb NOT NULL
);
CREATE TABLE "public"."lang" (
  "lang_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "name" text NOT NULL,
  "enable" bool NOT NULL DEFAULT true,
  "lang" text NOT NULL,
  "sort" int8 NOT NULL,
  "code" text NOT NULL
);
CREATE TABLE "public"."redirect" (
  "redirect_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "url" text NOT NULL,
  "permanently" bool NOT NULL,
  "redirect" text NOT NULL
);
CREATE TABLE "public"."role" (
  "role_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "name" jsonb NOT NULL,
  "desc" jsonb NOT NULL
);
CREATE TABLE "public"."route" (
  "route_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "url" text NOT NULL,
  "controller_id" int8 NOT NULL,
  "params" text,
  "lang_id" int8
);
CREATE TABLE "public"."session" (
  "session_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "user_id" int8 NOT NULL,
  "lang_id" int8 NOT NULL,
  "session" text NOT NULL,
  "data" bytea NOT NULL,
  "created" timestamptz(0) NOT NULL,
  "last" timestamptz(6) NOT NULL,
  "ip" text NOT NULL,
  "user_agent" text NOT NULL
);
CREATE TABLE "public"."setting" (
  "setting_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "key" text NOT NULL,
  "data" jsonb NOT NULL
);
CREATE TABLE "public"."user" (
  "user_id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY,
  "enable" bool NOT NULL DEFAULT false,
  "lang_id" int8 NOT NULL,
  "create" timestamptz(6) NOT NULL,
  "protect" bool NOT NULL,
  "role_id" int8 NOT NULL
);

INSERT INTO "public"."access" VALUES (1, 0, 't', 1);
INSERT INTO "public"."controller" VALUES (1, 'index', '', '', '[]');
INSERT INTO "public"."controller" VALUES (2, 'index', 'index', 'index', '[]');
INSERT INTO "public"."controller" VALUES (3, 'index', 'index', 'not_found', '[]');
INSERT INTO "public"."controller" VALUES (4, 'index', 'article', 'index', '[]');
INSERT INTO "public"."lang" VALUES (0, 'English', 't', 'en', 0, 'us');
INSERT INTO "public"."lang" VALUES (1, 'Українська', 't', 'uk', 1, 'ua');
INSERT INTO "public"."role" VALUES (0, '["Guest", "Гість"]', '["Unregister user", "Незареєстрований користувач"]');
INSERT INTO "public"."role" VALUES (1, '["Administrator", "Адміністратор"]', '["Full rules", "Повні права"]');
INSERT INTO "public"."role" VALUES (2, '["Registered user", "Зареєстрований користувач"]', '["Restricted access", "Обмежений доступ"]');
INSERT INTO "public"."route" VALUES (1, '/index.html', 2, NULL, NULL);
INSERT INTO "public"."route" VALUES (2, '/404.html', 3, NULL, NULL);
INSERT INTO "public"."route" VALUES (3, '/about.html', 4, 'about', NULL);
INSERT INTO "public"."route" VALUES (4, '/travel.html', 4, 'travel', NULL);
INSERT INTO "public"."route" VALUES (5, '/article.html', 4, 'article', NULL);
INSERT INTO "public"."route" VALUES (6, '/contact.html', 4, 'contact', NULL);
INSERT INTO "public"."route" VALUES (7, '/terms.html', 4, 'contact', NULL);
INSERT INTO "public"."route" VALUES (8, '/policy.html', 4, 'contact', NULL);
INSERT INTO "public"."user" VALUES (0, 't', 0, '2023-01-01 00:00:00+02', 't', 0);

SELECT setval('"public"."access_access_id_seq"', 1, true);
SELECT setval('"public"."controller_controller_id_seq"', 4, true);
SELECT setval('"public"."lang_lang_id_seq"', 1, true);
SELECT setval('"public"."role_role_id_seq"', 2, true);
SELECT setval('"public"."route_route_id_seq"', 8, true);

CREATE INDEX "access_access_idx" ON "public"."access" USING btree (
  "access" "pg_catalog"."bool_ops" ASC NULLS LAST
);
CREATE INDEX "access_controller_id_idx" ON "public"."access" USING btree (
  "controller_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
CREATE UNIQUE INDEX "access_role_id_controller_id_idx" ON "public"."access" USING btree (
  "role_id" "pg_catalog"."int8_ops" ASC NULLS LAST,
  "controller_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
CREATE INDEX "access_role_id_idx" ON "public"."access" USING btree (
  "role_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
ALTER TABLE "public"."access" ADD CONSTRAINT "access_pkey" PRIMARY KEY ("access_id");
CREATE UNIQUE INDEX "controller_module_class_action_idx" ON "public"."controller" USING btree (
  "module" "pg_catalog"."text_ops" ASC NULLS LAST,
  "class" "pg_catalog"."text_ops" ASC NULLS LAST,
  "action" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE INDEX "controller_module_class_action_idx1" ON "public"."controller" USING btree (
  "module" "pg_catalog"."text_ops" ASC NULLS LAST,
  "class" "pg_catalog"."text_ops" ASC NULLS LAST,
  "action" "pg_catalog"."text_ops" ASC NULLS LAST
) WHERE length(module) > 0 AND length(class) > 0 AND length(action) > 0;
ALTER TABLE "public"."controller" ADD CONSTRAINT "controller_expr_ch" CHECK (length(module) = 0 AND length(class) = 0 AND length(action) = 0 OR length(module) > 0 AND length(class) = 0 AND length(action) = 0 OR length(module) > 0 AND length(class) > 0 AND length(action) = 0 OR length(module) > 0 AND length(class) > 0 AND length(action) > 0);
ALTER TABLE "public"."controller" ADD CONSTRAINT "controller_pkey" PRIMARY KEY ("controller_id");
CREATE INDEX "lang_code_idx" ON "public"."lang" USING btree (
  "code" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE INDEX "lang_enable_idx" ON "public"."lang" USING btree (
  "enable" "pg_catalog"."bool_ops" ASC NULLS LAST
);
CREATE UNIQUE INDEX "lang_lang_code_idx" ON "public"."lang" USING btree (
  "lang" "pg_catalog"."text_ops" ASC NULLS LAST,
  "code" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE INDEX "lang_lang_idx" ON "public"."lang" USING btree (
  "lang" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE INDEX "lang_name_idx" ON "public"."lang" USING btree (
  "name" "pg_catalog"."text_ops" ASC NULLS LAST
);
ALTER TABLE "public"."lang" ADD CONSTRAINT "lang_pkey" PRIMARY KEY ("lang_id");
CREATE UNIQUE INDEX "redirect_url_idx" ON "public"."redirect" USING btree (
  "url" "pg_catalog"."text_ops" ASC NULLS LAST
);
ALTER TABLE "public"."redirect" ADD CONSTRAINT "redirect_pkey" PRIMARY KEY ("redirect_id");
CREATE UNIQUE INDEX "role_name_idx" ON "public"."role" USING btree (
  "name" "pg_catalog"."jsonb_ops" ASC NULLS LAST
);
ALTER TABLE "public"."role" ADD CONSTRAINT "role_pkey" PRIMARY KEY ("role_id");
CREATE INDEX "route_controller_id_idx" ON "public"."route" USING btree (
  "controller_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
CREATE INDEX "route_lang_id_idx" ON "public"."route" USING btree (
  "lang_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
CREATE INDEX "route_params_idx" ON "public"."route" USING btree (
  "params" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE UNIQUE INDEX "route_url_idx" ON "public"."route" USING btree (
  "url" "pg_catalog"."text_ops" ASC NULLS LAST
);
ALTER TABLE "public"."route" ADD CONSTRAINT "route_pkey" PRIMARY KEY ("route_id");
CREATE UNIQUE INDEX "session_session_idx" ON "public"."session" USING btree (
  "session" "pg_catalog"."text_ops" ASC NULLS LAST
);
CREATE INDEX "session_user_id_idx" ON "public"."session" USING btree (
  "user_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
ALTER TABLE "public"."session" ADD CONSTRAINT "session_pkey" PRIMARY KEY ("session_id");
CREATE UNIQUE INDEX "setting_key_idx" ON "public"."setting" USING btree (
  "key" "pg_catalog"."text_ops" ASC NULLS LAST
);
ALTER TABLE "public"."setting" ADD CONSTRAINT "setting_pkey" PRIMARY KEY ("setting_id");
CREATE INDEX "user_enable_idx" ON "public"."user" USING btree (
  "enable" "pg_catalog"."bool_ops" ASC NULLS LAST
);
CREATE INDEX "user_lang_id_idx" ON "public"."user" USING btree (
  "lang_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
CREATE INDEX "user_protect_idx" ON "public"."user" USING btree (
  "protect" "pg_catalog"."bool_ops" ASC NULLS LAST
);
CREATE INDEX "user_role_id_idx" ON "public"."user" USING btree (
  "role_id" "pg_catalog"."int8_ops" ASC NULLS LAST
);
ALTER TABLE "public"."user" ADD CONSTRAINT "user_pkey" PRIMARY KEY ("user_id");
ALTER TABLE "public"."access" ADD CONSTRAINT "access_controller_id_fkey" FOREIGN KEY ("controller_id") REFERENCES "public"."controller" ("controller_id") ON DELETE NO ACTION ON UPDATE NO ACTION;
ALTER TABLE "public"."access" ADD CONSTRAINT "access_role_id_fkey" FOREIGN KEY ("role_id") REFERENCES "public"."role" ("role_id") ON DELETE NO ACTION ON UPDATE NO ACTION;
ALTER TABLE "public"."route" ADD CONSTRAINT "route_controller_id_fkey" FOREIGN KEY ("controller_id") REFERENCES "public"."controller" ("controller_id") ON DELETE NO ACTION ON UPDATE NO ACTION;
ALTER TABLE "public"."route" ADD CONSTRAINT "route_lang_id_fkey" FOREIGN KEY ("lang_id") REFERENCES "public"."lang" ("lang_id") ON DELETE NO ACTION ON UPDATE NO ACTION;
ALTER TABLE "public"."session" ADD CONSTRAINT "session_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."user" ("user_id") ON DELETE NO ACTION ON UPDATE NO ACTION;
ALTER TABLE "public"."user" ADD CONSTRAINT "user_lang_id_fkey" FOREIGN KEY ("lang_id") REFERENCES "public"."lang" ("lang_id") ON DELETE NO ACTION ON UPDATE NO ACTION;
ALTER TABLE "public"."user" ADD CONSTRAINT "user_role_id_fkey" FOREIGN KEY ("role_id") REFERENCES "public"."role" ("role_id") ON DELETE NO ACTION ON UPDATE NO ACTION;

