import { useEffect, useMemo, useRef } from "react";
import { Link, Navigate, useParams } from "react-router-dom";
import hljs from "highlight.js";
import "highlight.js/styles/atom-one-dark.css";
import "../subapps/blog/blog.css";
import { renderPostBody } from "../subapps/blog/renderPost.js";
import { findPost, postPath, posts } from "../subapps/blog/posts.js";
import { asPosts, useEntries } from "../lib/siteEntries";

function formatDate(value) {
  return String(value || "").slice(0, 10);
}

export function BlogIndexPage() {
  const { items: articleEntries } = useEntries("article");
  const published = asPosts(articleEntries);
  const visible = [...published, ...posts];
  useEffect(() => {
    const previous = document.title;
    document.title = "文章 · Spume";
    return () => {
      document.title = previous;
    };
  }, []);

  return (
    <section className="site-blog">
      <div className="site-blog-wrap">
        <p className="site-blog-kicker">Notes</p>
        <h1>文章</h1>
        <p className="site-blog-lead">学习笔记、项目记录和随手写下的东西。</p>
        <div className="site-blog-list">
          {visible.length ? null : <p className="site-blog-lead">还没有文章。</p>}
          {visible.map((post) => (
            <Link key={post.slug} className="site-blog-card" to={postPath(post.slug)}>
              <div className="site-blog-meta">
                <time dateTime={formatDate(post.date)}>{formatDate(post.date)}</time>
                {post.categories.map((category) => <span key={category}>{category}</span>)}
              </div>
              <h2>{post.title}</h2>
              <p>{post.description}</p>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}

export function BlogPostPage() {
  const params = useParams();
  const splat = params["*"] || "";
  const { items: articleEntries, loaded } = useEntries("article");
  const published = asPosts(articleEntries);
  const post = useMemo(() => {
    const fromFile = findPost(splat);
    if (fromFile) return fromFile;
    const slug = decodeURIComponent(splat || "").split("/").filter(Boolean).at(-1) || "";
    return published.find((item) => item.slug === slug.replace(/\.html$/, ""));
  }, [published, splat]);
  const articleRef = useRef(null);
  const html = useMemo(() => (post ? renderPostBody(post.body) : ""), [post]);

  useEffect(() => {
    const previous = document.title;
    document.title = post ? `${post.title} · Spume` : "文章 · Spume";
    return () => {
      document.title = previous;
    };
  }, [post]);

  useEffect(() => {
    if (!post?.id) return;
    const marker = `viewed:article:${post.id}`;
    try {
      if (sessionStorage.getItem(marker)) return;
      sessionStorage.setItem(marker, "1");
    } catch {
      /* 存储不可用时不去重 */
    }
    fetch("/api/client/view", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ kind: "article", id: String(post.id) }),
    }).catch(() => {});
  }, [post]);

  useEffect(() => {
    articleRef.current?.querySelectorAll("pre code").forEach((block) => {
      hljs.highlightElement(block);
    });
  }, [html]);

  if (!splat) return <Navigate to="/blog" replace />;

  if (!post && !loaded) {
    return (
      <section className="site-blog">
        <div className="site-blog-wrap">
          <p className="site-blog-lead">正在读取文章。</p>
        </div>
      </section>
    );
  }

  if (!post) {
    return (
      <section className="site-blog">
        <div className="site-blog-wrap">
          <Link className="site-blog-back" to="/blog">返回文章</Link>
          <h1>没有找到这篇文章</h1>
        </div>
      </section>
    );
  }

  return (
    <article className="site-blog">
      <div className="site-blog-wrap">
        <Link className="site-blog-back" to="/blog">返回文章</Link>
        <p className="site-blog-kicker">{post.categories.join(" / ") || "Article"}</p>
        <h1 className="site-blog-title">{post.title}</h1>
        <div className="site-blog-meta">
          <time dateTime={formatDate(post.date)}>{formatDate(post.date)}</time>
          {post.tags.map((tag) => <span key={tag}>{tag}</span>)}
        </div>
        <div
          ref={articleRef}
          className="bt-prose"
          dangerouslySetInnerHTML={{ __html: html }}
        />
      </div>
    </article>
  );
}
