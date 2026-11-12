import { useEffect } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "../subapps/journal/src/components/Layout";
import { I18nProvider } from "../subapps/journal/src/i18n/I18nContext";
import { ArchivesPage } from "../subapps/journal/src/pages/ArchivesPage";
import { ArticlesPage } from "../subapps/journal/src/pages/ArticlesPage";
import { HomePage } from "../subapps/journal/src/pages/HomePage";
import { PostPage } from "../subapps/journal/src/pages/PostPage";
import "virtual:journal-scoped.css";

export default function JournalPage() {
  useEffect(() => {
    return () => {
      document.title = "Spume";
    };
  }, []);

  return (
    <div className="journal-root">
      <I18nProvider>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<HomePage />} />
            <Route path="articles" element={<ArticlesPage />} />
            <Route path="archives" element={<ArchivesPage />} />
            <Route path="post/:slug" element={<PostPage />} />
            <Route path="*" element={<Navigate to="/build" replace />} />
          </Route>
        </Routes>
      </I18nProvider>
    </div>
  );
}
