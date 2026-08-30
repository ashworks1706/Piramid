import "../globals.css";
import type { ReactNode } from "react";
import { Navbar } from "../../components/Navbar";
import { buildSearchIndex } from "../../lib/blogs";

export default function BlogsLayout({ children }: { children: ReactNode }) {
  const searchEntries = buildSearchIndex();

  return (
    <div className="min-h-screen bg-[#05070d] text-zinc-100">
      <Navbar searchEntries={searchEntries} />
      <main className="mx-auto max-w-6xl px-4 sm:px-6 py-8 lg:py-10">{children}</main>
    </div>
  );
}
