import { presetWind3 } from "@unocss/preset-wind3";
import { defineConfig } from "unocss";

export default defineConfig({
  presets: [presetWind3()],
  content: {
    pipeline: {
      include: [/apps\/[^/]+\/src\/.*\.[jt]sx?$/, /packages\/ui\/src\/.*\.[jt]sx?$/],
    },
  },
});
