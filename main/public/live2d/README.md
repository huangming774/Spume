# Live2D 看板娘（自托管）

主页左下角的看板娘，全部资源自托管在本目录，不依赖外部 CDN。

## 组成与来源

| 内容 | 来源 | 许可 |
| --- | --- | --- |
| `dist/autoload.js`、`waifu.css`、`waifu-tips.js`、`live2d.min.js` | [stevenjoezhang/live2d-widget](https://github.com/stevenjoezhang/live2d-widget)（npm `live2d-widgets@1.0.1`） | GPL-3.0，见 `LICENSE-widget.txt` |
| `dist/waifu-tips.json` | 本站重写（官方默认文案不适配本站且含不宜内容，未采用；原文件留档 `waifu-tips.default.json`） | 随 widget GPL-3.0 |
| `dist/waifu-site.css` | 本站适配样式（层级/移动端） | 随站点 |
| `model/` 下全部模型 | [fghrsh/live2d_api](https://github.com/fghrsh/live2d_api)（Cubism 2.1） | 仓库代码 MIT（`LICENSE-models.txt`）；**模型本身版权归原作者** |
| `model_list.json` | 本站按已下载模型子集编写 | 随站点 |

## 已收录模型

- bilibili-live/22、bilibili-live/33（含多套换装）
- ShizukuTalk/shizuku-48、shizuku-pajama（官方示例模型）
- KantaiCollection/murakumo
- HyperdimensionNeptunia/neptune_classic（依赖的 nepnep/general 共享表情/动作/音效已一并收录）

## 许可注意

- Live2D 模型、图片、动作数据版权均属于原作者，**仅供研究学习展示，不得用于商业用途**。
- Cubism 2 Core（`live2d.min.js`）为 Live2D 官方 Redistributable Code，使用需遵守
  [Live2D SDK License Agreement (Public)](https://docs.google.com/document/d/10tz1WrycskzGGBOhrAfGiTSsgmyFy8D9yHx9r_PsN8I/)。
- widget 代码 GPL-3.0；本站点仓库为 MIT，`dist/` 内第三方文件保持其原许可，不随仓库许可变更。

## 本站改动

- `autoload.js`：`live2d_path` 指向 `/live2d/dist/`，`cdnPath` 指向 `/live2d/`（模型清单与模型全部站内）；
  屏宽 <768px 不加载；开启拖动。
- 入口在 `main/index.html`（SPA 外壳，路由切换不刷新，天然满足 PJAX 场景要求）。
- `sw.js` 已排除 `/live2d/`，避免 Service Worker 缓存旧资源。
