import client from "prom-client";

export const PromClient = client;

export const answeredQueryCounter = new PromClient.Counter({
	name: "answered_queries",
	help: "Number of answered queries"
});

export const responseTimesHistogram = new PromClient.Histogram({
	name: "response_times",
	help: "Response times of queries",
	buckets: [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000]
});
