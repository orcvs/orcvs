import type { Config } from "@jest/types"

const SECONDS = 1000; 

const testTimeout = 30 * SECONDS;

const config: Config.InitialOptions = {
  // preset: "ts-jest/presets/default-esm",  
  preset: "ts-jest",
  testEnvironment: "node",
  verbose: true,
  automock: false,
  testTimeout,
  roots: [
    "tests",
  ],
  modulePathIgnorePatterns: [
    "<rootDir>/node_modules/",
  ],
  clearMocks: true,
}
export default config


