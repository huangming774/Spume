---
title: 我是如何借助 AI 学习数据库 SQL 的：一次"一页一页讲"的学习实践
date: 2026-05-24 23:25:00
tags: [AI, SQL, 学习方法, 数据库]
categories: [学习笔记]
cover: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/20260524-232506-ee241f.png
---

<img src="https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/20260524-232506-ee241f.png" alt="封面" style="width:100%">

最近我在学习数据库课程中的《第5章 数据操作语句》。这部分内容主要围绕 SQL 查询展开，包括 `SELECT`、`WHERE`、`LIKE`、`ORDER BY`、聚合函数等。

这篇博客不是单纯整理 SQL 知识点，而是想记录一种新的学习方式：

{% note info %}
借助 AI，把课件变成一个可以互动的 **"私人老师"**。
{% endnote %}

我让 AI 按照课件一页一页讲解，每讲完一页，我可以随时提问；遇到例题时，AI 先不给答案，而是让我自己写 SQL，再判断对错并讲解原因。通过这种方式，原本静态的 PPT 变成了一个可对话、可练习、可纠错的学习过程。

---

## 一、我的学习规则设计

一开始，我给 AI 设定了明确的学习规则：

{% folding blue, 9 条学习规则 %}
1. 先阅读课件，告诉我总共多少页。
2. 我说"开始"后，从第一页开始讲。
3. 每次只讲一页。
4. 我说 {% span green, "1" %}，表示这一页懂了，进入下一页。
5. 遇到例题时，AI 先只给题目，不给答案。
6. 我先自己写 SQL。
7. AI 判断我写得对不对。
8. 如果错了，AI 讲清楚错在哪里、为什么错、怎么改。
9. 我觉得重要的地方，让 AI 记录为"注意点"，最后统一总结。
{% endfolding %}

这种规则的好处是：**学习节奏完全由我控制**，不会一下子被大量内容淹没。

---

## 二、AI 的角色不是"直接给答案"，而是"陪我思考"

以前看课件时，遇到例题我可能会直接看答案，感觉自己懂了，但真正写的时候还是容易错。

这次我要求 AI 遇到题目时不要直接讲答案，而是先把题目给我。

例如课件中有一道题：

{% note default %}
查询价格低于 50 的图书的书名和价格。
{% endnote %}

我先自己写：

```sql
SELECT bname, price
FROM books
WHERE price < 50;
```

AI 判断正确后，再告诉我每一部分对应的含义：

- `SELECT bname, price`：查询书名和价格
- `FROM books`：从图书表中查询
- `WHERE price < 50`：筛选价格低于 50 的图书

这种方式比直接看答案更有效，因为我经历了 **"自己想 → 自己写 → 被反馈"** 的过程。

---

## 三、AI 能及时纠正我的误区

在学习过程中，我经常会写出一些"看起来差不多，但实际有问题"的 SQL。

例如有一道题是：

{% note default %}
查询计算机学院全体学生的姓名。
{% endnote %}

我一开始写成了：

```sql
SELECT sname
FROM students
WHERE press = '计算机学院';
```

AI 立刻指出：`press` 通常表示出版社，不是学院。学生所在学院应该用类似 `college` 的字段。

这让我意识到：

{% note danger %}
SQL 不只是语法问题，还要理解字段的含义。
{% endnote %}

后来我也和 AI 讨论了：如果考试或作业没给表结构怎么办？最后形成了一个学习原则：

{% tip info %}
如果题目没有明确表结构，字段名意思相近、语法逻辑正确，可以先认为思路正确；但正式写题时，必须以题目给出的表结构为准。
{% endtip %}

---

## 四、通过追问，把模糊概念问清楚

这次学习中，我发现 AI 最大的价值不是"讲课"，而是**可以随时追问**。

比如我看到：

```sql
WHERE sname LIKE '张%'
```

我问 AI：

> {% span blue, 张% 这个百分号是什么？ %}

AI 解释说，`%` 是 `LIKE` 里的通配符，表示任意多个字符。所以 `LIKE '张%'` 表示姓名以"张"开头，可以匹配"张三""张小明""张伟"等。

后来我又继续问：

> {% span blue, 如果本来应该写等于，我写成 LIKE 可以吗？ %}

AI 进一步解释：

- `WHERE college = '计算机学院'` 是精确匹配。
- `WHERE college LIKE '计算机学院'` 在没有 `%` 或 `_` 的情况下，通常结果类似，但不推荐这样写。精确匹配应该用 `=`，模糊匹配才用 `LIKE`。

这种连续追问让我不是只记住写法，而是真正理解了它们的区别。

---

## 五、AI 帮我把"易错点"沉淀成注意点

学习过程中，我会主动让 AI 把一些容易错的地方记为"注意点"。

例如我在学习 `NOT IN` 和多个"不等于"条件时，容易把 `AND` 和 `OR` 搞混。

AI 帮我总结成：

```sql
WHERE press NOT IN ('商务印书馆', '清华大学出版社')
```

等价于：

```sql
WHERE press != '商务印书馆'
  AND press != '清华大学出版社';
```

**而不是用 `OR`**。

后来遇到：

```sql
WHERE SID NOT LIKE '%2'
  AND SID NOT LIKE '%3'
  AND SID NOT LIKE '%5';
```

AI 又提醒我：

{% note danger %}
多个"不是……"条件要用 **AND**，不能用 **OR**。
{% endnote %}

这个总结对我很有帮助，因为它不是孤立知识点，而是从我做错的地方提炼出来的。

---

## 六、AI 不只是讲知识，还能帮我建立学习节奏

这次学习中，我用 {% span green, "1" %} 作为进入下一页的信号。

例如：

- AI 讲完第 30 页 → 我提问 → AI 解答 → 我确认懂了 → 我发送 {% span green, "1" %} → AI 继续讲第 31 页。

这种方式让我感觉自己不是被动听课，而是在**主动控制学习进度**。

尤其是遇到难点时，我可以停下来一直问，直到搞懂为止；遇到简单页，也可以快速进入下一页。

---

## 七、一次典型的互动学习流程

我的学习流程大概是这样的：

### 1. AI 讲解课件页面

例如讲 `ORDER BY`：

```sql
ORDER BY price ASC   -- 按价格升序排列
ORDER BY price DESC  -- 按价格降序排列
```

### 2. AI 给出例题

{% note default %}
查询 "TP" 类图书的详细信息，查询结果按价格升序排序。
{% endnote %}

### 3. 我先写 SQL

```sql
SELECT *
FROM books
WHERE categories = 'TP'
ORDER BY price ASC;
```

### 4. AI 判断

AI 判断我的逻辑正确，并提醒如果按课件字段名，`categories` 可能应为 `category`。

### 5. AI 补充总结

`ASC` 可以省略，因为默认是升序：

```sql
ORDER BY price;
```

### 6. 我把重要点加入注意点

例如：`ORDER BY` 可以按多个字段排序，前面的字段优先级更高。

---

## 八、这次学习中形成的几个关键注意点

下面这些不是完整知识点整理，而是我在学习过程中**真正容易错、真正问过**的问题。

{% folding red, 1. 日期相差天数不要直接相减 %}

在 MySQL 中，计算两个日期相差天数，推荐使用：

```sql
TIMESTAMPDIFF(DAY, borrow_time, return_time)
```

不要简单写成：

```sql
return_time - borrow_time
```

因为日期直接相减可能不会得到真正的天数差。
{% endfolding %}

{% folding red, 2. OR 连接多个条件时，每个条件都要写完整 %}

{% u 错误写法： %}

```sql
WHERE press = '商务印书馆'
   OR '清华大学出版社'
   OR '人民邮电出版社';
```

{% u 正确写法： %}

```sql
WHERE press = '商务印书馆'
   OR press = '清华大学出版社'
   OR press = '人民邮电出版社';
```

如果都是同一个字段匹配多个值，更推荐：

```sql
WHERE press IN ('商务印书馆', '清华大学出版社', '人民邮电出版社');
```
{% endfolding %}

{% folding red, 3. 多个"不是……"条件要用 AND %}

例如查询学号最后一位不是 2、3、5：

```sql
WHERE SID NOT LIKE '%2'
  AND SID NOT LIKE '%3'
  AND SID NOT LIKE '%5';
```

不能写成多个 `OR`。
{% endfolding %}

{% folding red, 4. 判断空值不能用 = NULL %}

{% u 错误写法： %}

```sql
WHERE return_time = NULL;
```

{% u 正确写法： %}

```sql
WHERE return_time IS NULL;
```

判断非空：

```sql
WHERE return_time IS NOT NULL;
```
{% endfolding %}

{% folding red, 5. COUNT(*) 和 COUNT(列名) 不一样 %}

- `COUNT(*)` — 统计表中有多少行。
- `COUNT(return_time)` — 统计 `return_time` 这一列中**非空值**的个数。

所以如果 `return_time` 有 `NULL`，`COUNT(return_time)` 不会统计那一行。
{% endfolding %}

---

## 九、这种学习方式给我的感受

这次借助 AI 学习 SQL，我最大的感受是：

{% note success %}
AI 最适合做一个 **"可以随时暂停、随时追问、随时纠错"** 的学习搭子。
{% endnote %}

它不是简单地把答案告诉我，而是在我写错时指出问题，在我不理解时换一种方式解释，在我觉得重要时帮我总结注意点。

和自己看课件相比，这种方式更像是一场互动式学习。

我不只是"看懂了"，而是在不断经历：

{% span blue, 看题 → 思考 → 写 SQL → 被纠错 → 追问 → 总结 %}

这个过程让我对知识点的记忆更深。

---

## 十、下一步学习计划

目前我已经学到了 SQL 单表查询中的聚合函数，包括：

{% span red, COUNT %}  {% span blue, SUM %}  {% span green, AVG %}  {% span orange, MAX %}  {% span purple, MIN %}

下一步准备继续学习：

{% note info %}
**GROUP BY** — 也就是分组统计。
{% endnote %}

接下来我还会继续保持这种学习方式：

- 一页一页学
- 先自己做题
- 错了再让 AI 讲
- 把易错点整理成注意点
- 最后形成自己的学习笔记

---

{% note success %}
这次实践让我发现，AI 不只是一个答案工具，更可以成为一个帮助我建立学习过程、发现薄弱点、整理知识结构的 **学习助手**。
{% endnote %}
