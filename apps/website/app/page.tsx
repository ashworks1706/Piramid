import Link from "next/link";
import { Navbar } from "../components/Navbar";

export default function Home() {
  return (
    <div className="min-h-screen bg-[#05070d] text-slate-100">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_30%_10%,rgba(99,102,241,0.12),transparent_40%),radial-gradient(ellipse_at_75%_80%,rgba(14,165,233,0.07),transparent_40%)] pointer-events-none" />

      <Navbar />

      <main className="relative mx-auto max-w-3xl px-6 py-24 flex flex-col gap-20">
        {/* Hero */}
        <section className="space-y-7">
          <h1 className="text-4xl sm:text-5xl font-semibold leading-[1.15] tracking-tight text-white">
            Inference Engine for Retrieval-Augmented Systems
          </h1>

          <p className="text-lg text-slate-400 leading-relaxed max-w-xl">
            Standard RAG stuffs retrieved text into the prompt. Piramid feeds it
            into the model&apos;s attention. One Rust binary runs retrieval and
            inference in a single process&mdash;retrieved neighbors attend
            through cross-attention layers during the forward pass, not as
            concatenated context tokens.
          </p>

          <p className="text-sm text-slate-500 leading-relaxed max-w-xl">
            Built on custom models that already know
            how to attend over your knowledge base.
          </p>

          <div className="flex flex-wrap gap-3 pt-1">
            <Link
              href="/blogs"
              className="rounded-full bg-indigo-500 text-white px-5 py-2 text-sm font-semibold shadow-lg shadow-indigo-500/25 hover:bg-indigo-400 transition-colors"
            >
              Read the blog
            </Link>
            <a
              href="https://github.com/ashworks1706/piramid"
              className="rounded-full border border-white/15 px-5 py-2 text-sm font-semibold text-slate-300 hover:border-white/40 hover:text-white transition-colors"
            >
              View on GitHub
            </a>
            <code className="rounded-full border border-white/10 bg-white/[0.04] px-5 py-2 text-sm font-mono text-slate-400 select-all tracking-tight">
              cargo install piramid
            </code>
          </div>
        </section>

        {/* How it works */}
        <section className="space-y-5">
          <h2 className="text-sm font-semibold uppercase tracking-widest text-slate-500">
            How it works
          </h2>

          <div className="grid gap-6 sm:grid-cols-3">
            <div className="space-y-2">
              <p className="text-sm font-semibold text-white">Retrieve</p>
              <p className="text-sm text-slate-400 leading-relaxed">
                Query hits the built-in vector index. ANN search returns the
                nearest neighbor chunks.
              </p>
            </div>
            <div className="space-y-2">
              <p className="text-sm font-semibold text-white">Fuse</p>
              <p className="text-sm text-slate-400 leading-relaxed">
                Retrieved chunks are encoded and injected into the
                transformer&apos;s cross-attention layers during the forward
                pass.
              </p>
            </div>
            <div className="space-y-2">
              <p className="text-sm font-semibold text-white">Generate</p>
              <p className="text-sm text-slate-400 leading-relaxed">
                The model generates grounded in retrieved knowledge without
                burning context window on stuffed text.
              </p>
            </div>
          </div>
        </section>

        {/* Why */}
        <section className="space-y-5">
          <h2 className="text-sm font-semibold uppercase tracking-widest text-slate-500">
            Why
          </h2>

          <div className="space-y-4 text-sm text-slate-400 leading-relaxed">
            <p>
              <span className="text-white font-medium">No prompt stuffing.</span>{" "}
              An 8k context model can reason over a million-vector knowledge base
              because retrieval goes through attention, not through the context
              window.
            </p>
            <p>
              <span className="text-white font-medium">Single process.</span>{" "}
              No network hop between retrieval and generation. Index and model
              live in one Rust binary.
            </p>
            <p>
              <span className="text-white font-medium">
                Pre-built models.
              </span>{" "}
              Ships with models already retrofitted for retrieval-augmented
              inference. Bring your data, not your training pipeline.
            </p>
          </div>
        </section>

        {/* Footer */}
        <footer className="flex flex-wrap gap-x-5 gap-y-1 pb-4 text-sm text-slate-500">
          <Link
            href="/blogs"
            className="hover:text-slate-300 transition-colors"
          >
            Blog
          </Link>
          <a
            href="https://github.com/ashworks1706/piramid"
            className="hover:text-slate-300 transition-colors"
          >
            GitHub
          </a>
          <a
            href="https://crates.io/crates/piramid"
            className="hover:text-slate-300 transition-colors"
          >
            crates.io
          </a>
          <span className="ml-auto">piramid © 2026</span>
        </footer>
      </main>
    </div>
  );
}
