"use client";

import { useEffect, useState } from "react";
import { CLI_FRAMES } from "../lib/cli-frames";

/** Milliseconds per frame. Eleven frames, so the build runs in a little over half a second. */
const FRAME_MS = 55;
const LAST = CLI_FRAMES.length - 1;

/**
 * Plays the same ASCII animation the `piramid` binary prints on startup.
 *
 * Rendered as preformatted text rather than an image, so it stays crisp at any zoom and costs no
 * network request. It holds on the final frame.
 *
 * Stepped with setTimeout rather than requestAnimationFrame. rAF is smoother in principle, but it
 * does not advance in every environment that renders the page, and a stalled clock here leaves a
 * half-drawn logo on screen. Each timeout is scheduled against elapsed wall time rather than a
 * fixed delay, so a slow tick catches up on the next one instead of dragging the whole sequence.
 */
export function CliAnimation() {
  const [frame, setFrame] = useState(0);
  const settled = frame >= LAST;

  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      // Scheduled rather than set here: a synchronous setState inside an effect body cascades a
      // render. One tick later the reader sees the finished logo with no build-up.
      const id = window.setTimeout(() => setFrame(LAST), 0);
      return () => window.clearTimeout(id);
    }

    const started = Date.now();
    let timer = 0;

    const step = () => {
      const next = Math.min(Math.floor((Date.now() - started) / FRAME_MS), LAST);
      setFrame(next);
      if (next < LAST) {
        const due = started + (next + 1) * FRAME_MS - Date.now();
        timer = window.setTimeout(step, Math.max(due, 0));
      }
    };

    timer = window.setTimeout(step, FRAME_MS);
    return () => window.clearTimeout(timer);
  }, []);

  return (
    <pre
      aria-label="Piramid"
      className={`cli-animation select-none${settled ? " is-settled" : ""}`}
    >
      {CLI_FRAMES[frame]}
    </pre>
  );
}
