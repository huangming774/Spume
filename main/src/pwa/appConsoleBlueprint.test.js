import test from "node:test";
import assert from "node:assert/strict";

import {
  buildMobileArticleDraft,
  getNotificationTotal,
  ownerConsoleAvatars,
  ownerConsoleModules,
  ownerConsoleNotifications,
  ownerConsoleScreens,
  publishSteps,
} from "./appConsoleBlueprint.js";

test("restores the owner controller screen map from the prototype", () => {
  assert.deepEqual(
    ownerConsoleScreens.map((screen) => screen.id),
    ["home", "article", "drafts", "gallery", "moments", "friend", "inbox", "ai"],
  );
  assert.equal(ownerConsoleScreens[0].title, "站长工作台");
  assert.equal(ownerConsoleScreens[1].title, "发布文章");
});

test("keeps the prototype task modules and notification badge content", () => {
  assert.deepEqual(
    ownerConsoleModules.map((module) => module.title),
    ["发布文章", "相册图片", "发布碎语", "增加友链", "留言收件箱"],
  );
  assert.deepEqual(
    ownerConsoleNotifications.map((notification) => notification.title),
    ["留言板新留言", "朋友页新评论", "友链申请提醒"],
  );
  assert.equal(getNotificationTotal(ownerConsoleNotifications), 3);
});

test("defines avatar choices and the original publish automation steps", () => {
  assert.equal(ownerConsoleAvatars.length, 5);
  assert.equal(ownerConsoleAvatars[0].initial, "TC");
  assert.deepEqual(publishSteps, [
    "检查内容",
    "上传图片",
    "生成页面",
    "发布到网站",
    "检查线上结果",
  ]);
});

test("generates a mobile AI article draft for review before publishing", () => {
  const draft = buildMobileArticleDraft("今天把图片和文字交给 AI，由 AI 写文章。");

  assert.equal(draft.title, "用 AI 代理完成一次移动端发文");
  assert.match(draft.body, /这篇文章由手机端 AI 代理/);
  assert.match(draft.body, /站长审核通过后再进入自动化发布流程/);
});
