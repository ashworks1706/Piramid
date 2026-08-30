import type { ReactNode } from "react";
import { DocsSidebar } from "../../../components/DocsSidebar";
import { DocsSidebarMobile } from "../../../components/DocsSidebarMobile";
import { buildSidebar } from "../../../lib/blogs";

/**
 * Posts get the section sidebar; the blog index does not, because there it would just repeat the
 * listing that is already the page.
 */
export default function PostLayout({ children }: { children: ReactNode }) {
  const sidebar = buildSidebar();

  return (
    <div className="flex flex-col gap-6 lg:flex-row lg:gap-8">
      <DocsSidebarMobile sections={sidebar} />
      <aside className="hidden lg:block w-64 flex-shrink-0">
        <DocsSidebar sections={sidebar} />
      </aside>
      <article className="min-w-0 flex-1">{children}</article>
    </div>
  );
}
