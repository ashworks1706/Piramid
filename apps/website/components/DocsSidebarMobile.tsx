"use client";

import { useState } from "react";
import type { SidebarSection } from "../lib/blogs";
import { DocsSidebar } from "./DocsSidebar";

type Props = {
  sections: SidebarSection[];
};

export function DocsSidebarMobile({ sections }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <div className="lg:hidden">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-center justify-between rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-semibold text-white shadow-sm shadow-slate-900/30 transition hover:border-indigo-400/60"
      >
        <span>Blog navigation</span>
        <span aria-hidden="true" className="text-lg">
          {open ? "×" : "☰"}
        </span>
      </button>
      {open ? (
        <div className="mt-3">
          <DocsSidebar sections={sections} sticky={false} className="w-full" />
        </div>
      ) : null}
    </div>
  );
}
