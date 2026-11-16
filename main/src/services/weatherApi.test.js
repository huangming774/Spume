import test from "node:test";
import assert from "node:assert/strict";

import { fetchWeather } from "./weatherApi.js";

test("fetchWeather returns weather payload with defaults for missing optional fields", async () => {
  let requestedUrl = "";
  const result = await fetchWeather({
    fetchImpl: async (url) => {
      requestedUrl = url;
      return {
        ok: true,
        json: async () => ({
          city: "杭州",
          region: "浙江",
          temperature: 21.5,
          weatherCode: 2,
          weatherDescription: "局部多云",
          windSpeed: 8.2,
          isDay: true,
        }),
      };
    },
  });

  assert.equal(requestedUrl, "/api/weather");
  assert.deepEqual(result, {
    city: "杭州",
    region: "浙江",
    temperature: 21.5,
    weatherCode: 2,
    weatherDescription: "局部多云",
    windSpeed: 8.2,
    isDay: true,
  });
});

test("fetchWeather fills defaults for omitted fields", async () => {
  const result = await fetchWeather({
    fetchImpl: async () => ({
      ok: true,
      json: async () => ({ temperature: 10, weatherCode: 0 }),
    }),
  });

  assert.deepEqual(result, {
    city: "",
    region: "",
    temperature: 10,
    weatherCode: 0,
    weatherDescription: "",
    windSpeed: undefined,
    isDay: undefined,
  });
});

test("fetchWeather throws readable errors for failed responses", async () => {
  await assert.rejects(
    () =>
      fetchWeather({
        fetchImpl: async () => ({
          ok: false,
          json: async () => ({ message: "天气信息暂时拿不到" }),
        }),
      }),
    /天气信息暂时拿不到/,
  );
});
