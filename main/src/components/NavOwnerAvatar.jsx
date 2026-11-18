import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import clsx from "clsx";
import { LuLogIn } from "react-icons/lu";
import {
  defaultSiteAvatarUrl,
  fetchSiteProfile,
  invalidateSiteProfile,
  onAvatarUpdated,
} from "../lib/siteAvatar.js";

/** 导航栏右上角的站长头像：取公开站点头像，后台更换后实时刷新。 */
const NavOwnerAvatar = ({ className }) => {
  const navigate = useNavigate();
  const [avatar, setAvatar] = useState(defaultSiteAvatarUrl);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const load = (force) => {
      fetchSiteProfile({ force }).then((profile) => {
        if (!cancelled) {
          setAvatar(profile.avatar);
          setFailed(false);
        }
      });
    };
    load(false);
    const unsubscribe = onAvatarUpdated(() => {
      invalidateSiteProfile();
      load(true);
    });
    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, []);

  const showImage = avatar && !failed;

  return (
    <button
      type="button"
      className={clsx(
        className,
        "flex h-9 w-9 items-center justify-center overflow-hidden rounded-full"
      )}
      aria-label="站长登录"
      title="站长登录"
      onClick={() => navigate("/app")}
    >
      {showImage ? (
        <img
          src={avatar}
          alt="站长头像"
          className="h-full w-full object-cover"
          onError={() => setFailed(true)}
        />
      ) : (
        <LuLogIn aria-hidden="true" />
      )}
    </button>
  );
};

export default NavOwnerAvatar;
