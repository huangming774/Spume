# 内容写作说明

文章内容由 `content/articles/` 下的 Markdown 生成，运行 `python build_content.py` 会更新 React 数据文件。

## 目录

- `content/articles/` - 长文，生成文章列表、详情页数据和归档时间线数据。

## 构建

```bash
pip install -r requirements.txt
python build_content.py
```

## 文章 frontmatter 示例

```yaml
---
date: 2026-05-15
slug: my-post
title_zh: 中文标题
title_en: English title
excerpt_zh: 摘要
excerpt_en: Summary
words: 3200
reads: 256
minutes: 12
---

正文 Markdown（中文）……

---en---

English Markdown body…
```
