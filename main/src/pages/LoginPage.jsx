import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

/** 仅打开右下角 AI 面板；登录须在面板内点击「登录 / 注册」 */
const LoginPage = () => {
  const navigate = useNavigate();

  useEffect(() => {
    window.dispatchEvent(
      new CustomEvent("blog-ai-open", { detail: { openAuth: true, mode: "login" } }),
    );
    navigate("/", { replace: true });
  }, [navigate]);

  return null;
};

export default LoginPage;
