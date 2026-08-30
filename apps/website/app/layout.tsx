import type { Metadata } from "next";
import { JetBrains_Mono } from "next/font/google";
import localFont from "next/font/local";
import "./globals.css";
import "katex/dist/katex.min.css";

// Self-hosted by next/font, so there is no render-blocking request to Google.
const mono = JetBrains_Mono({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-mono",
});

// Google's `latin` subset stops at U+00FF plus a little punctuation, so it carries none of the
// box-drawing or block characters the logo is drawn with. Those glyphs fell through to whatever
// the OS happened to pick, and the two ranges can land on *different* fallbacks — which is what
// smeared the wordmark on Windows while leaving the pyramid roughly intact. JetBrains Mono itself
// has all 160 of them at the same 600/1000 advance as its ASCII, so this ships exactly that block.
// `display: "block"` because the fallback is the bug: better a beat of nothing than a beat of
// broken art.
const blocks = localFont({
  src: "./fonts/jetbrains-mono-blocks.woff2",
  display: "block",
  variable: "--font-blocks",
  declarations: [{ prop: "unicode-range", value: "U+2500-259F" }],
});

export const metadata: Metadata = {
  title: {
    template: "%s | Piramid",
    default: "Piramid – Inference engine for RAG",
  },
  description:
    "Piramid is a single-binary vector database in Rust: mmap and WAL durability, HNSW, IVF and flat indexes, filter-aware search, and embedding providers. Built toward running retrieval and inference in one process.",
  keywords: [
    "vector database",
    "rust",
    "low latency",
    "HNSW",
    "IVF",
    "flat index",
    "embeddings",
    "RAG",
    "agentic",
    "similarity search",
  ],
  authors: [{ name: "ashworks1706" }],
  creator: "ashworks1706",
  publisher: "ashworks1706",
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://piramiddb.com",
    title: "Piramid – Inference engine for RAG",
    description:
      "A single-binary vector database in Rust, built toward running retrieval and inference in one process.",
    siteName: "Piramid",
    images: [
      {
        url: "../public/logo_dark.png",
        width: 1200,
        height: 630,
        alt: "Piramid Vector Database",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Piramid – Inference engine for RAG",
    description:
      "A single-binary vector database in Rust, built toward running retrieval and inference in one process.",
    images: ["../public/logo_dark.png"],
    creator: "@piramiddb",
  },
  metadataBase: new URL("https://piramiddb.com"),
  alternates: {
    canonical: "/",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    // Browser extensions add attributes to <html> before hydration; this only covers this
    // element, so a real mismatch further down still reports.
    <html lang="en" className={`dark ${mono.variable} ${blocks.variable}`} suppressHydrationWarning>
      <body className="antialiased">{children}</body>
    </html>
  );
}
