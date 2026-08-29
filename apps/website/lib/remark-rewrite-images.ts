import path from "path";

/**
 * A remark plugin factory that rewrites relative image paths in markdown files
 * to the absolute /assets/... URLs Next.js serves from public/.
 *
 * Markdown under content/blogs/ references images relatively, e.g.
 * `../../assets/blogs/lsm.png`, which keeps the files previewable in an editor.
 * Next.js only serves static files from public/, so those relative paths have
 * to become absolute URLs.
 *
 * The rewrite matches on the `assets/` path segment rather than resolving
 * against the filesystem. Filesystem resolution tied this plugin to the exact
 * depth of the markdown file and to a copy of the images living outside the
 * app; matching the segment keeps it working wherever the content sits.
 *
 * Images are served from apps/website/public/assets/blogs/, which is the single
 * canonical copy. They were previously duplicated at the repo root, and the two
 * copies had drifted: public/ was missing lsm.png entirely, so the image in the
 * storage post did not render.
 *
 * Usage:
 *   remarkPlugins: [remarkGfm, remarkRewriteImages()]
 */
export function remarkRewriteImages() {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return () => (tree: any) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function walk(node: any) {
      if (
        node.type === "image" &&
        typeof node.url === "string" &&
        !node.url.startsWith("http") &&
        !node.url.startsWith("/")
      ) {
        // Normalise separators, then keep everything from the `assets/` segment
        // onward: `../../assets/blogs/lsm.png` becomes `/assets/blogs/lsm.png`.
        const normalised = node.url.split(path.sep).join("/");
        const marker = normalised.indexOf("assets/");
        if (marker !== -1) {
          node.url = "/" + normalised.slice(marker);
        }
      }
      if (Array.isArray(node.children)) {
        for (const child of node.children) walk(child);
      }
    }
    walk(tree);
  };
}
