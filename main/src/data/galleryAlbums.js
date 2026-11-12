import { cosAsset } from "../lib/cosAsset.js";

const COS = cosAsset("");
const MISAKA = `${COS}/%E6%97%8B%E8%BD%AC%E7%9B%B8%E5%86%8C%E5%BE%A1%E5%9D%82%E7%BE%8E%E7%90%B4`;
const AI_BLOG = `${COS}/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87`;

export const galleryAlbums = [
  {
    id: "misaka",
    title: "御坂美琴",
    eyebrow: "Anime Archive 01",
    description: "常盘台的电击公主，被集中收进主站 Gallery 的第一本相册。",
    tone: "from-[#EAF6FF] via-[#F7F1FF] to-[#FFF5FA]",
    accent: "#7C5CFF",
    cover: `${MISAKA}/915cdb4184d77e9d06e789755f30f792.jpg`,
    images: [
      `${MISAKA}/4535f8677a92b484bd9320d27fdc5517.jpg`,
      `${MISAKA}/49b59d762bf5cd0c10d18537d2d8af36.jpg`,
      `${MISAKA}/0eb95d8658244e51fbcb0da56acb8427.jpg`,
      `${MISAKA}/4bc63bca979d7348d500f27685b2501d.jpg`,
      `${MISAKA}/6ab90bbd7326d788c80b7014768bd0c7.jpg`,
      `${MISAKA}/7a83b1f88d1b571dbed005cdd1707f7a.jpg`,
      `${MISAKA}/915cdb4184d77e9d06e789755f30f792.jpg`,
      `${MISAKA}/970e4914909bb82a70edbac46bd34f47.jpg`,
      `${MISAKA}/b1fcbdeeb1cc22bf00cf99bd3a186e9a.jpg`,
      `${MISAKA}/fb9073623eb3e3bce01de71f916b3ab9.jpg`,
    ],
  },
  {
    id: "oregairu",
    title: "春物",
    eyebrow: "Anime Archive 02",
    description: "我的青春恋爱物语果然有问题，原 blog 相册内容迁移到主站统一展示。",
    tone: "from-[#FFF8F1] via-[#FFEAF4] to-[#F6FBFF]",
    accent: "#FF8FAB",
    cover: `${AI_BLOG}/8850f2efda188bd1dabdd68b7cb47ddd.jpg`,
    images: [
      `${AI_BLOG}/8850f2efda188bd1dabdd68b7cb47ddd.jpg`,
      `${AI_BLOG}/476eacc0536c04604d8881f9c4782c7a.jpg`,
      `${AI_BLOG}/fdff0b6c899061246aa4f24781413fb6.jpg`,
      `${AI_BLOG}/f2c4e89bfc6471f917fccccb2cc97150.jpg`,
      `${AI_BLOG}/02eff38f6ec5d32a1175a638933c4a2d.jpg`,
      `${AI_BLOG}/dc43312bd7868afc6e6b66f801df1930.jpg`,
      `${AI_BLOG}/4004b6caa5f5fdec9bd42904ec641657.jpg`,
      `${AI_BLOG}/280029aa5be7b45731188a1c375ab34b.jpg`,
      `${AI_BLOG}/1c6bfb08af427f9bbf83103a9558df6f.jpg`,
      `${AI_BLOG}/0e7e10e2acdb398941c10735a791918d.jpg`,
    ],
  },
  {
    id: "tangwulin",
    title: "唐舞麟",
    eyebrow: "Anime Archive 03",
    description: "金龙月语与龙王传说相关图片，作为第三本统一相册保留。",
    tone: "from-[#F6FBFF] via-[#ECFFF7] to-[#FFF8F1]",
    accent: "#2FAE91",
    cover: `${COS}/e75ae60d917f2b0e99aaa1530017657.jpg`,
    images: [
      `${COS}/e75ae60d917f2b0e99aaa1530017657.jpg`,
      `${COS}/0b226672166ec46d1c2c6e61e6f616b.jpg`,
      `${COS}/3990de0688d71af8e8fe16c4d0726a2.jpg`,
      `${COS}/4a615619c07eee9bd06cb4049d458dd.jpg`,
      `${COS}/8f15208c9340078a307f210d2d60d76.jpg`,
      `${COS}/abcc9128742e729a437dab9a1e76071.jpg`,
      `${COS}/b04e03255cb7d789313c96fbadffc88.jpg`,
      `${COS}/d1f89b0618399985bf1256a93dfda3c.jpg`,
      `${COS}/91e2ed43c281a2aaeac59edd6a11195.jpg`,
      `${COS}/fd88946f9ebd15192179fb9ade6c212.jpg`,
    ],
  },
];

export const getGalleryAlbum = (id) =>
  galleryAlbums.find((album) => album.id === id);
