"use client";

import Link from "next/link";
import type { BlogMeta } from "../lib/blogs";

type Props = {
  prev?: BlogMeta;
  next?: BlogMeta;
  wide?: boolean;
};

export function DocsPager({ prev, next, wide = false }: Props) {
  if (!prev && !next) return null;

  const containerClass = wide
    ? "mb-4 grid w-full grid-cols-1 gap-3 sm:grid-cols-2"
    : "mb-4 flex flex-wrap gap-3 text-sm";

  const linkBase =
    "inline-flex items-center gap-2 rounded-full border px-4 py-2 text-zinc-200 shadow-sm shadow-zinc-900/20 transition w-full";

  return (
    <div className={containerClass}>
      {prev && (
        <Link
          href={
            prev.slug.join("/") === "index"
              ? "/blogs"
              : `/blogs/${prev.slug.join("/")}`
          }
          className={`${linkBase} border-white/10 bg-white/5 hover:border-cyan-400/60 hover:text-white`}
        >
          <span aria-hidden="true">←</span>
          <span className="truncate max-w-[180px] sm:max-w-[260px]">
            {prev.title}
          </span>
        </Link>
      )}
      {next && (
        <Link
          href={
            next.slug.join("/") === "index"
              ? "/blogs"
              : `/blogs/${next.slug.join("/")}`
          }
          className={`${linkBase} border-cyan-400/50 bg-cyan-500/10 text-zinc-100 shadow-cyan-950/30 hover:border-cyan-300/70`}
        >
          <span className="truncate max-w-[180px] sm:max-w-[260px]">
            {next.title}
          </span>
          <span aria-hidden="true">→</span>
        </Link>
      )}
    </div>
  );
}
