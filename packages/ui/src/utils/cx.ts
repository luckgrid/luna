import type { ClassValue } from "clsx";
import { clsx } from "clsx";

export function cx(...inputs: ClassValue[]) {
  return clsx(inputs);
}
