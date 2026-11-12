---
title: Claude Code 手机版实战：多轮修正 Word 文档直到完美提交
date: 2026-05-14 22:00:00
tags: [Claude Code, AI工具, 效率提升]
categories: [AI编程]
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/Screenshot_20260514_204759.jpg
description: 在手机上用 Claude Code 直接编辑 .docx 的 XML，多轮迭代修改数据库实验报告直到完美提交的全过程。
---

## 场景

今天在手机上完成了一次完整的文档修改工作流：一份数据库实验报告需要做格式规范化，通过 {% span blue, Claude Code %} 直接在手机上操作 {% u .docx %} 文件，经过两轮迭代修改，最终达到提交标准。

{% note primary %}
整个过程全部在手机上完成：接收文件 → 解压 XML → Claude 分析修改 → 打包 → 发送回微信。全程没有打开电脑上的 Word，甚至没有离开聊天界面。
{% endnote %}

---

## 任务背景

手头有份《数据库原理及应用 实验二》的实验报告，上一轮已经改过主要问题（{% span green, NVARCHAR、N 前缀、触发器集合写法、权限管理 %}），但还有几处格式细节需要打磨：

| 序号 | 位置 | 问题 |
|------|------|------|
| 1 | 第（二）1 存储过程 | {% span red, GO %} 放在了注释后面 |
| 2 | 第（二）2 存储过程 | {% span red, WHERE 子句 %} 缺少分号 |
| 3 | 第（四）1 安全管理 | {% span red, CREATE LOGIN %} 后缺 GO |
| 4 | 第（四）3 授权 | {% span red, GRANT %} 前缺 USE/GO |

---

## 修正过程

### 第一轮：四个格式问题

**1. GO 从注释后移到 END; 之后**

{% label 修改前 red %}
```sql
END;

-- 调用示例：
GO
EXEC 查询指定年龄学生 @年龄 = 20;
```

{% label 修改后 green %}
```sql
END;
GO

-- 调用示例：
EXEC 查询指定年龄学生 @年龄 = 20;
```

**2. WHERE 子句补分号**

`WHERE 学生.所在系 = @系名` → `WHERE 学生.所在系 = @系名;`

**3. CREATE LOGIN 后加 GO，USE 后加 GO**

```sql
CREATE LOGIN u1 WITH PASSWORD = '123456', CHECK_POLICY = OFF;
CREATE LOGIN u2 WITH PASSWORD = '123456', CHECK_POLICY = OFF;
CREATE LOGIN u3 WITH PASSWORD = '123456', CHECK_POLICY = OFF;
GO

USE StudentsXXXX;
GO

CREATE USER u1 FOR LOGIN u1;
```

**4. GRANT 前补 USE StudentsXXXX + GO**

```sql
USE StudentsXXXX;
GO

GRANT SELECT ON 课程 TO u1;
GRANT SELECT ON 选课 TO u1;
GRANT INSERT ON 课程 TO u2;
GO
```

### 第二轮：两个小细节

{% folding blue, 点击展开第二轮修改详情 %}

第一轮改完后审查，又发现两个可优化的地方：

**1. 存储过程内部语句补分号**
```sql
PRINT N'年龄错误';   -- 补分号
WHERE 年龄 = @年龄;   -- 补分号
```

**2. 四.5 节点 USE 后加 GO**
```sql
USE StudentsXXXX;
GO  -- 新增

GRANT CREATE TABLE TO u3;
GRANT ALTER ON SCHEMA::dbo TO u3;
GO
```

{% endfolding %}

---

## 技术原理

{% tip info %}
很多人不知道：{% span red, .docx 文件本质上是一个 ZIP 压缩包 %}，里面是一堆 XML 文件。所以可以直接解压 → 改 XML → 打包回去，完全不需要 Word。
{% endtip %}

{% tabs docx-workflow %}

<!-- tab 解压 -->
```bash
python unpack.py 报告.docx unpacked/
```
解压后得到 `word/document.xml` 等 15 个 XML 文件。

<!-- tab 编辑 XML -->
直接对 `document.xml` 做字符串替换：

{% label 补分号 blue %}
```xml
<!-- 改前 -->
<w:t>WHERE 学生.所在系 = @系名</w:t>
<!-- 改后 -->
<w:t>WHERE 学生.所在系 = @系名;</w:t>
```

{% label 插 GO 段 blue %}
```xml
<!-- 在 USE StudentsXXXX; 后插入 -->
<w:p>
  <w:r><w:t>GO</w:t></w:r>
</w:p>
```

<!-- tab 打包 -->
```bash
python pack.py unpacked/ 报告.docx
```
验证无误后，直接发回微信。

{% endtabs %}

---

## 最终效果

{% progress 100%, blue, 修正完成度 %}

| 检查项 | 状态 |
|--------|------|
| SQL 逻辑正确 | {% span green, ✓ %} |
| GO 批处理规范 | {% span green, ✓ %} |
| 触发器集合写法 | {% span green, ✓ %} |
| 权限管理完整 | {% span green, ✓ %} |
| 分号/格式细节 | {% span green, ✓ %} |
| 结论说明严谨 | {% span green, ✓ %} |

{% note success %}
**最终评价：** 这版作业逻辑正确、SQL 规范、说明合理，可以提交。
{% endnote %}

---

## 工具链

整个流程依赖三个核心工具：

{% tabs toolchain %}
<!-- tab Claude Code -->
**Claude Code CLI** — 移动端的 AI 编程终端

- 在手机上运行 `claude` 命令直接与 AI 对话
- 可以读取文件、编辑代码、执行 Python 脚本
- 支持自定义 Skills 扩展能力（如 docx 处理）

本场景中 Claude Code 负责：分析 XML 内容 → 定位 SQL 语法问题 → 生成修正方案 → 执行 Python 打包

<!-- tab Python 脚本 -->
**unpack.py / pack.py** — .docx ↔ XML 互转

```python
# unpack.py - 解压 docx 为可编辑的 XML 目录
import zipfile, os, sys
zip_path = sys.argv[1]
out_dir = sys.argv[2]
with zipfile.ZipFile(zip_path, 'r') as z:
    z.extractall(out_dir)

# pack.py - 将修改后的 XML 目录重新打包为 docx
import zipfile, os, sys
in_dir = sys.argv[1]
out_path = sys.argv[2]
with zipfile.ZipFile(out_path, 'w', zipfile.ZIP_DEFLATED) as zout:
    for root, dirs, files in os.walk(in_dir):
        for f in files:
            full = os.path.join(root, f)
            arcname = os.path.relpath(full, in_dir)
            zout.write(full, arcname)
```

{% label 关键点 blue %} 必须是 ZIP_DEFLATED 压缩，否则 Word 打不开。

<!-- tab cc-connect -->
**cc-connect** — 手机与电脑/微信之间的桥梁

- `cc-connect send --file` 发送文件到微信
- 双向传输，手机上改完直接发回

{% endtabs %}

---

## 小结

这次体验证明了在手机上用 AI 完成文档精细修改完全可行：

{% checkbox checked green, .docx = ZIP + XML，直接编辑比打开 Word 更快 %}
{% checkbox checked green, 多轮迭代修正，每轮都有明确的验收标准 %}
{% checkbox checked green, 全程手机完成，从修改到发送一气呵成 %}
{% checkbox , 以后实验报告、作业文档都可以这么改 %}

{% span cyan, 工具组合：%}{% span blue, Claude Code %} + {% span red, Python 脚本 %} + {% span green, cc-connect %}

试想一下这个流程：老师在微信上发来一份实验报告模板，你在手机上用 Claude Code 直接打开、修改、打包、发回——整个过程不超过十分钟。不需要传到电脑、不需要打开 Office、不需要来回切换设备。这就是 AI CLI 工具在移动端的真正价值所在。

{% note success %}
**适用场景：** 实验报告、课程作业、社团文档、实习周报——凡是格式规范化的 .docx 文档修改，都可以用这套流程解决。
{% endnote %}
