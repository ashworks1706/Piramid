import type { Metadata } from "next";
import { JetBrains_Mono } from "next/font/google";
import "./globals.css";
import "katex/dist/katex.min.css";

// Self-hosted by next/font, so there is no render-blocking request to Google.
const mono = JetBrains_Mono({
  subsets: ["latin"],
  display: "swap",
  variable: "--font-mono",
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
    // suppressHydrationWarning covers only this element's own attributes, not its children.
    // Extensions commonly add attributes to <html>, and React reports that as a mismatch it
    // cannot patch even though the app itself renders identically on both sides.
    <html lang="en" className={`dark ${mono.variable}`} suppressHydrationWarning>
      <body className="antialiased">{children}</body>
    </html>
  );
}
