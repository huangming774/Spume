import { useState } from "react";
import { Link } from "react-router-dom";
import { friendCards } from "../data/friendCards";
import { notifyEntriesChanged, useEntries } from "../lib/siteEntries";
import {
  deleteOwnerEntry,
  publishOwnerAbout,
  publishOwnerBili,
  publishOwnerProject,
  uploadOwnerAsset,
} from "../services/ownerApi";

function RecordList({ items, empty, onDelete, busyId }) {
  if (!items.length) return <p className="owner-record-empty">{empty}</p>;
  return (
    <ul className="owner-record-list">
      {items.map((item) => (
        <li key={item.id}>
          <span>{item.label}</span>
          {item.viewTo ? (
            <Link className="owner-secondary" to={item.viewTo}>
              查看
            </Link>
          ) : null}
          {item.viewHref ? (
            <a className="owner-secondary" href={item.viewHref} target="_blank" rel="noreferrer">
              查看
            </a>
          ) : null}
          <button type="button" onClick={() => onDelete(item)} disabled={busyId === item.id}>
            {busyId === item.id ? "删除中..." : "删除"}
          </button>
        </li>
      ))}
    </ul>
  );
}

async function storeImage(file, kind) {
  const data = await uploadOwnerAsset(file, { kind });
  const url = data.item?.url || "";
  if (!url) throw new Error("图片没有写入 S3");
  return url;
}

export function ArticleLibrary() {
  const { items: articles } = useEntries("article");
  const { items: journals } = useEntries("journal");
  const blogItems = articles.map((item) => ({
    id: item.id,
    kind: "article",
    label: `文章 · ${item.title}`,
    viewTo: `/blog/${item.id}`,
  }));
  const journalItems = journals.map((item) => ({
    id: item.id,
    kind: "journal",
    label: `建站日志 · ${item.title}`,
    viewTo: `/build/post/${item.id}`,
  }));
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");

  const remove = async (item) => {
    if (!window.confirm(`删除「${item.label}」？`)) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, item.kind);
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <section className="owner-panel owner-glass owner-library">
      <div className="owner-panel-title">
        <h2>已发布内容</h2>
      </div>
      <RecordList items={blogItems} empty="还没有文章。" onDelete={remove} busyId={busyId} />
      <RecordList items={journalItems} empty="还没有建站日志。" onDelete={remove} busyId={busyId} />
      {error ? <p className="owner-record-error">{error}</p> : null}
    </section>
  );
}

export function GalleryLibrary() {
  const { items: stored } = useEntries("gallery");
  const items = stored.map((item) => ({
    id: item.id,
    kind: "gallery",
    label: `${item.title || "相册"} · ${item.imageUrl}`,
    viewTo: `/gallery/${item.extra?.albumId || item.id}`,
  }));
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");

  const remove = async (item) => {
    if (!window.confirm("删除这张相册图片？")) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, "gallery");
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "相册图片删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <section className="owner-panel owner-glass owner-library">
      <div className="owner-panel-title">
        <h2>已发布图片</h2>
      </div>
      <RecordList items={items} empty="还没有相册图片。" onDelete={remove} busyId={busyId} />
      {error ? <p className="owner-record-error">{error}</p> : null}
    </section>
  );
}

export function MomentLibrary() {
  const { items: stored } = useEntries("moment");
  const items = stored.map((item) => ({
    id: item.id,
    kind: "moment",
    label: `${item.extra?.year || ""}.${item.extra?.date || ""} · ${item.title}`,
    viewTo: "/glimmer",
  }));
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");

  const remove = async (item) => {
    if (!window.confirm(`删除碎语「${item.label}」？`)) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, "moment");
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "碎语删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <section className="owner-panel owner-glass owner-library">
      <div className="owner-panel-title">
        <h2>已发布碎语</h2>
      </div>
      <RecordList items={items} empty="还没有碎语。" onDelete={remove} busyId={busyId} />
      {error ? <p className="owner-record-error">{error}</p> : null}
    </section>
  );
}

export function AboutPanel() {
  const { items: stored } = useEntries("about");
  const items = stored.map((entry) => ({ id: entry.id, label: entry.title, viewTo: "/about" }));
  const [title, setTitle] = useState("");
  const [text, setText] = useState("");
  const [image, setImage] = useState("");
  const [busy, setBusy] = useState(false);
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const upload = async (event) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    setBusy(true);
    setError("");
    try {
      setImage(await storeImage(file, "about"));
    } catch (cause) {
      setError(cause.message || "图片上传失败");
    } finally {
      setBusy(false);
    }
  };

  const publish = async () => {
    setBusy(true);
    setError("");
    try {
      await publishOwnerAbout({ title, text, image });
      notifyEntriesChanged();
      setNotice("已写入关于我。正文在数据库，图片地址指向 S3。");
      setTitle("");
      setText("");
      setImage("");
    } catch (cause) {
      setError(cause.message || "关于我发布失败");
    } finally {
      setBusy(false);
    }
  };

  const remove = async (item) => {
    if (!window.confirm(`删除「${item.label}」？`)) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, "about");
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "关于我删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <div className="owner-form-shell owner-glass">
      <div>
        <div className="owner-field">
          <label htmlFor="aboutTitle">标题</label>
          <input id="aboutTitle" value={title} onChange={(event) => setTitle(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="aboutText">正文</label>
          <textarea id="aboutText" value={text} onChange={(event) => setText(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="aboutImage">图片</label>
          <input id="aboutImage" value={image} placeholder="上传后会填入 S3 地址" onChange={(event) => setImage(event.target.value)} />
        </div>
        <div className="owner-quick-line">
          <label className="owner-secondary" htmlFor="aboutUpload">
            {busy ? "处理中..." : "上传图片到 S3"}
          </label>
          <input id="aboutUpload" type="file" accept="image/*" hidden onChange={upload} />
          <button type="button" className="owner-primary" onClick={publish} disabled={busy}>
            添加
          </button>
        </div>
        {error ? <p className="owner-record-error">{error}</p> : null}
        {notice ? (
          <p className="owner-record-empty">
            {notice} <Link to="/about">查看关于我</Link>
          </p>
        ) : null}
        <RecordList items={items} empty="还没有关于我的内容。" onDelete={remove} busyId={busyId} />
      </div>
    </div>
  );
}

export function BiliPanel() {
  const { items: stored } = useEntries("bili");
  const items = stored.map((video) => ({ id: video.id, label: video.title, viewTo: "/bili" }));
  const [bvid, setBvid] = useState("");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [duration, setDuration] = useState("");
  const [cover, setCover] = useState("");
  const [busy, setBusy] = useState(false);
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const upload = async (event) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    setBusy(true);
    setError("");
    try {
      setCover(await storeImage(file, "bili"));
    } catch (cause) {
      setError(cause.message || "封面上传失败");
    } finally {
      setBusy(false);
    }
  };

  const publish = async () => {
    setBusy(true);
    setError("");
    try {
      await publishOwnerBili({ bvid, title, description, cover, duration });
      notifyEntriesChanged();
      setNotice("已写入哔哩视频列表。");
      setBvid("");
      setTitle("");
      setDescription("");
      setDuration("");
      setCover("");
    } catch (cause) {
      setError(cause.message || "哔哩视频添加失败");
    } finally {
      setBusy(false);
    }
  };

  const remove = async (item) => {
    if (!window.confirm(`删除视频「${item.label}」？`)) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, "bili");
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "哔哩视频删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <div className="owner-form-shell owner-glass">
      <div>
        <div className="owner-field">
          <label htmlFor="biliBvid">BV 号</label>
          <input id="biliBvid" value={bvid} placeholder="BV1xx411c7mD" onChange={(event) => setBvid(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="biliTitle">标题</label>
          <input id="biliTitle" value={title} onChange={(event) => setTitle(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="biliDescription">简介</label>
          <textarea id="biliDescription" value={description} onChange={(event) => setDescription(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="biliDuration">时长</label>
          <input id="biliDuration" value={duration} placeholder="03:21" onChange={(event) => setDuration(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="biliCover">封面</label>
          <input id="biliCover" value={cover} placeholder="上传后会填入 S3 地址" onChange={(event) => setCover(event.target.value)} />
        </div>
        <div className="owner-quick-line">
          <label className="owner-secondary" htmlFor="biliUpload">
            {busy ? "处理中..." : "上传封面到 S3"}
          </label>
          <input id="biliUpload" type="file" accept="image/*" hidden onChange={upload} />
          <button type="button" className="owner-primary" onClick={publish} disabled={busy}>
            添加
          </button>
        </div>
        {error ? <p className="owner-record-error">{error}</p> : null}
        {notice ? (
          <p className="owner-record-empty">
            {notice} <Link to="/bili">查看哔哩页</Link>
          </p>
        ) : null}
        <RecordList items={items} empty="还没有收录的视频。" onDelete={remove} busyId={busyId} />
      </div>
    </div>
  );
}

export function ProjectPanel() {
  const { items: stored } = useEntries("project");
  const items = stored.map((project) => ({
    id: project.id,
    label: project.title,
    viewTo: `/about/projects/${project.id}`,
  }));
  const [title, setTitle] = useState("");
  const [summary, setSummary] = useState("");
  const [url, setUrl] = useState("");
  const [tags, setTags] = useState("");
  const [cover, setCover] = useState("");
  const [busy, setBusy] = useState(false);
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const upload = async (event) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    setBusy(true);
    setError("");
    try {
      setCover(await storeImage(file, "project"));
    } catch (cause) {
      setError(cause.message || "封面上传失败");
    } finally {
      setBusy(false);
    }
  };

  const publish = async () => {
    setBusy(true);
    setError("");
    try {
      await publishOwnerProject({ title, summary, cover, url, tags });
      notifyEntriesChanged();
      setNotice("已写入星空馆。简介在数据库，封面地址指向 S3。");
      setTitle("");
      setSummary("");
      setUrl("");
      setTags("");
      setCover("");
    } catch (cause) {
      setError(cause.message || "星空馆项目添加失败");
    } finally {
      setBusy(false);
    }
  };

  const remove = async (item) => {
    if (!window.confirm(`删除项目「${item.label}」？`)) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, "project");
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "星空馆项目删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <div className="owner-form-shell owner-glass">
      <div>
        <div className="owner-field">
          <label htmlFor="projectTitle">标题</label>
          <input id="projectTitle" value={title} onChange={(event) => setTitle(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="projectSummary">简介</label>
          <textarea id="projectSummary" value={summary} onChange={(event) => setSummary(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="projectUrl">项目地址</label>
          <input id="projectUrl" value={url} placeholder="https://" onChange={(event) => setUrl(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="projectTags">标签</label>
          <input id="projectTags" value={tags} placeholder="用逗号分开" onChange={(event) => setTags(event.target.value)} />
        </div>
        <div className="owner-field">
          <label htmlFor="projectCover">封面</label>
          <input id="projectCover" value={cover} placeholder="上传后会填入 S3 地址" onChange={(event) => setCover(event.target.value)} />
        </div>
        <div className="owner-quick-line">
          <label className="owner-secondary" htmlFor="projectUpload">
            {busy ? "处理中..." : "上传封面到 S3"}
          </label>
          <input id="projectUpload" type="file" accept="image/*" hidden onChange={upload} />
          <button type="button" className="owner-primary" onClick={publish} disabled={busy}>
            添加
          </button>
        </div>
        {error ? <p className="owner-record-error">{error}</p> : null}
        {notice ? (
          <p className="owner-record-empty">
            {notice} <Link to="/projects">查看星空馆</Link>
          </p>
        ) : null}
        <RecordList items={items} empty="还没有星空馆项目。" onDelete={remove} busyId={busyId} />
      </div>
    </div>
  );
}

export function FriendLibrary() {
  const { items: stored } = useEntries("friend");
  const items = stored.map((entry) => ({
    id: entry.id,
    label: `${entry.title} · ${entry.extra?.url || ""}`,
    viewHref: entry.extra?.url || "",
  }));
  const [busyId, setBusyId] = useState("");
  const [error, setError] = useState("");

  const remove = async (item) => {
    if (!window.confirm(`删除友链「${item.label}」？朋友页会同步移除。`)) return;
    setBusyId(item.id);
    setError("");
    try {
      await deleteOwnerEntry(item.id, "friend");
      notifyEntriesChanged();
    } catch (cause) {
      setError(cause.message || "友链删除失败");
    } finally {
      setBusyId("");
    }
  };

  return (
    <section className="owner-panel owner-glass owner-library">
      <div className="owner-panel-title">
        <h2>已发布友链</h2>
      </div>
      <RecordList items={items} empty="还没有友链。可以在上方发布，或在收件箱一键通过申请。" onDelete={remove} busyId={busyId} />
      {friendCards.length ? (
        <>
          <p className="owner-record-empty">以下旧友链保存在源码里，如需移除请在仓库 friendCards.js 删除：</p>
          <ul className="owner-record-list">
            {friendCards.map((card) => (
              <li key={card.url}>
                <span>{card.name} · {card.url}</span>
                <a className="owner-secondary" href={card.url} target="_blank" rel="noreferrer">
                  查看
                </a>
              </li>
            ))}
          </ul>
        </>
      ) : null}
      {error ? <p className="owner-record-error">{error}</p> : null}
    </section>
  );
}
