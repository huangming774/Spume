import { Link } from "react-router-dom";
import { TiLocationArrow } from "react-icons/ti";
import { galleryAlbums } from "../data/galleryAlbums";

const GalleryPage = () => (
  <div className="relative min-h-screen overflow-hidden bg-[#fff8f1] pt-24 text-[#241322]">
    <div
      className="pointer-events-none fixed inset-0 -z-10 bg-[radial-gradient(circle_at_18%_18%,rgba(255,143,171,0.22),transparent_32%),radial-gradient(circle_at_82%_12%,rgba(124,92,255,0.14),transparent_28%),linear-gradient(135deg,#fff8f1_0%,#ffeef5_48%,#f6fbff_100%)]"
      aria-hidden="true"
    />
    <div className="pointer-events-none fixed left-1/2 top-24 -z-10 h-56 w-[78vw] -translate-x-1/2 rounded-full bg-white/55 blur-3xl" />

    <header className="container mx-auto px-5 pb-10 md:px-10">
      <p className="font-general text-xs uppercase tracking-[0.45em] text-[#b76e79]">
        Unified Gallery
      </p>
      <div className="mt-4 grid gap-6 md:grid-cols-[1.05fr_0.95fr] md:items-end">
        <h1 className="font-zentry text-5xl font-black uppercase leading-none text-[#241322] md:text-7xl">
          桃之夭夭
          <span className="block text-[#ff8fab]">Gallery</span>
        </h1>
        <p className="max-w-xl text-base leading-relaxed text-[#5f4b52]">
          一处安静的视觉档案。把喜欢的画面，分门别类地收进相册里，留作日后慢慢翻阅。
        </p>
      </div>
    </header>

    <section className="container mx-auto grid gap-6 px-5 pb-24 md:grid-cols-3 md:px-10">
      {galleryAlbums.map((album, index) => (
        <Link
          key={album.id}
          to={`/gallery/${album.id}`}
          className={`group relative min-h-[34rem] overflow-hidden rounded-[2rem] bg-gradient-to-br ${album.tone} p-4 text-[#241322] shadow-[0_28px_90px_rgba(95,75,82,0.14)] transition duration-500 hover:-translate-y-2 hover:shadow-[0_36px_110px_rgba(255,143,171,0.22)]`}
        >
          <div className="absolute inset-x-8 top-6 h-px bg-gradient-to-r from-transparent via-white/80 to-transparent" />
          <div className="relative h-[24rem] overflow-hidden rounded-[1.55rem] border border-white/70 bg-white/45 shadow-[0_18px_58px_rgba(95,75,82,0.12)]">
            <img
              src={album.cover}
              alt={album.title}
              className="size-full object-cover transition duration-700 group-hover:scale-105"
              loading={index === 0 ? "eager" : "lazy"}
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-white/15" />
            <span className="absolute left-4 top-4 rounded-full border border-white/40 bg-white/70 px-3 py-1 font-general text-[10px] uppercase tracking-[0.28em] text-[#5f4b52] backdrop-blur-md">
              {album.eyebrow}
            </span>
          </div>

          <div className="relative z-10 mt-5">
            <p className="font-general text-[10px] uppercase tracking-[0.36em] text-[#b76e79]">
              {String(album.images.length).padStart(2, "0")} Photos
            </p>
            <h2 className="mt-2 text-4xl font-black leading-none">
              {album.title}
            </h2>
            <p className="mt-3 text-sm leading-relaxed text-[#5f4b52]">
              {album.description}
            </p>
            <span
              className="mt-5 inline-flex items-center gap-2 rounded-full px-4 py-2 text-xs font-bold uppercase text-white"
              style={{ backgroundColor: album.accent }}
            >
              Open Album
              <TiLocationArrow />
            </span>
          </div>
        </Link>
      ))}
    </section>
  </div>
);

export default GalleryPage;
