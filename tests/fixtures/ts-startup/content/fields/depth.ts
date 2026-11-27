import { file } from "@origingame/engine/author";
import { samples } from "../../lib/depth.ts";

export default file.field({ dimensions: [2, 2, 1], channels: 1, values: samples });
