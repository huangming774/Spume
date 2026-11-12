import { Outlet, useLocation } from "react-router-dom";
import { useEffect } from "react";
import NavBar from "../components/Navbar";
import Footer from "../components/Footer";

const SiteLayout = () => {
  const { pathname } = useLocation();

  useEffect(() => {
    window.scrollTo(0, 0);
  }, [pathname]);

  return (
    <main className="relative min-h-screen w-full overflow-x-clip">
      <NavBar />
      <Outlet />
      <Footer />
    </main>
  );
};

export default SiteLayout;
