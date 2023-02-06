import pino from "pino";

const formatters = {
  bindings (bindings: any) {
    return { }
  },
  level (label: string, number: number) {
    return { level: label }
  }
}

const level = process.env.LOG_LEVEL === undefined ? 'info' : process.env.LOG_LEVEL.trim();

export const Logger = pino({
  level,
  formatters,
});
