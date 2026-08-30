"use client";

import { useEffect, useState } from "react";
import { CLI_FRAMES } from "../lib/cli-frames";

const FRAME_MS = 110;
const LAST = CLI_FRAMES.length - 1;

/**
 * Plays the same ASCII animation the `piramid` binary prints on startup.
 *
 * Rendered as preformatted text rather than an image, so it stays crisp at any zoom and costs no
 * network request. It holds on the final frame.
 */
export function CliAnimation() {
  const [frame, setFrame] = useState(0);

  useEffect(() => {
    const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (still) {
      // Scheduled rather than set here: a synchronous setState inside an effect body cascades a
      // render. One tick later the reader sees the finished logo with no build-up.
      const id = window.setTimeout(() => setFrame(LAST), 0);
      return () => window.clearTimeout(id);
    }

    let current = 0;
    const id = window.setInterval(() => {
      current += 1;
      setFrame(current);
      if (current >= LAST) window.clearInterval(id);
    }, FRAME_MS);

    return () => window.clearInterval(id);
  }, []);

  return (
    <pre aria-label="Piramid" className="cli-animation select-none">
      {CLI_FRAMES[frame]}
    </pre>
  );
}
