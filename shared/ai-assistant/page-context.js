(function (root, factory) {
  var api = factory();
  if (typeof module === "object" && module.exports) {
    module.exports = api;
  }
  root.BlogAIPageContext = api;
})(
  typeof window !== "undefined" ? window : globalThis,
  function () {
    "use strict";

    var DEFAULT_TEXT_LIMIT = 2200;
    var DEFAULT_HEADING_LIMIT = 10;

    function normalizeWhitespace(value) {
      return String(value || "").replace(/\s+/g, " ").trim();
    }

    function clipText(value, maxLength) {
      var text = normalizeWhitespace(value);
      var limit = maxLength || DEFAULT_TEXT_LIMIT;
      if (Array.from(text).length <= limit) return text;
      return Array.from(text).slice(0, limit).join("") + "...";
    }

    function cleanList(values, limit, itemLimit) {
      var seen = Object.create(null);
      var out = [];
      (values || []).forEach(function (value) {
        var text = clipText(value, itemLimit || 120);
        if (!text || seen[text]) return;
        seen[text] = true;
        out.push(text);
      });
      return out.slice(0, limit || DEFAULT_HEADING_LIMIT);
    }

    function inferSiteSection(path) {
      var pagePath = path || "/";
      if (pagePath.indexOf("/blog") === 0) return "hexo-blog";
      if (pagePath.indexOf("/build") === 0) return "growth-blog";
      if (pagePath.indexOf("/guestbook") === 0) return "guestbook";
      if (pagePath.indexOf("/bili") === 0) return "bili";
      if (pagePath.indexOf("/ai-traffic") === 0) return "ai-traffic";
      return "main";
    }

    function buildPageContext(input) {
      var pagePath = normalizeWhitespace(input && input.pagePath) || "/";
      return {
        pageUrl: clipText(input && input.pageUrl, 400),
        pageTitle: clipText(input && input.pageTitle, 120),
        pagePath: clipText(pagePath, 160),
        language: clipText(input && input.language, 30),
        siteSection:
          clipText(input && input.siteSection, 60) || inferSiteSection(pagePath),
        headings: cleanList(input && input.headings, DEFAULT_HEADING_LIMIT, 120),
        visibleText: clipText(input && input.visibleText, DEFAULT_TEXT_LIMIT),
      };
    }

    function pickContentRoot(doc) {
      if (!doc) return null;
      return (
        doc.querySelector("main") ||
        doc.querySelector("article") ||
        doc.querySelector("#post") ||
        doc.querySelector(".post-content") ||
        doc.querySelector("#root") ||
        doc.body
      );
    }

    function collectHeadings(doc) {
      if (!doc || !doc.querySelectorAll) return [];
      return Array.prototype.slice
        .call(doc.querySelectorAll("h1, h2, h3"))
        .map(function (node) {
          return node.textContent || "";
        });
    }

    function collectVisibleText(doc) {
      var rootNode = pickContentRoot(doc);
      if (!rootNode) return "";
      return rootNode.innerText || rootNode.textContent || "";
    }

    function collectPageContext(doc, loc) {
      var documentRef = doc || (typeof document !== "undefined" ? document : null);
      var locationRef = loc || (typeof location !== "undefined" ? location : {});
      var html = documentRef && documentRef.documentElement;
      return buildPageContext({
        pageUrl: locationRef.href || "",
        pageTitle:
          (documentRef && documentRef.title) ||
          (typeof document !== "undefined" ? document.title : ""),
        pagePath: locationRef.pathname || "/",
        language: (html && html.lang) || "",
        siteSection: inferSiteSection(locationRef.pathname || "/"),
        headings: collectHeadings(documentRef),
        visibleText: collectVisibleText(documentRef),
      });
    }

    function createChatPayload(message, context) {
      var pageContext = buildPageContext(context || {});
      return {
        message: message,
        pageUrl: pageContext.pageUrl,
        pageTitle: pageContext.pageTitle,
        pageContext: pageContext,
      };
    }

    return {
      buildPageContext: buildPageContext,
      collectPageContext: collectPageContext,
      createChatPayload: createChatPayload,
      normalizeWhitespace: normalizeWhitespace,
    };
  }
);
