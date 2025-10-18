"use client";

export default function Logistics() {
  const summary = `This system generates detailed, company-level climate impact reports by computing an ImpactScore, classifying risk, highlighting supporting evidence, and producing ranked mitigation recommendations with confidence, expected impact, and citations. Heavy processing—including scraping, parsing, embedding, multi-label classification, and feature aggregation—is performed offline by a Rust worker, while the Go API serves precomputed reports quickly, optionally invoking a GPU-backed C++ microservice for rescoring. All structured and vector data is stored in PostgreSQL with pgvector, and frequently accessed report JSON is cached in Redis for low-latency delivery. The frontend presents both admin and report interfaces, allowing users to explore predictions, supply chain impacts, market sentiment correlations, and cross-company benchmarking. Predictive models detect greenwashing, estimate future emissions, and provide actionable mitigation recommendations.`;

  return (
    <main className="h-[calc(100vh-80px)] bg-gradient-to-b from-emerald-50 to-white flex items-center justify-center p-8">
      <div className="max-w-4xl bg-white shadow-lg rounded-2xl p-8 border border-gray-100">
        <h1 className="text-3xl font-bold text-emerald-700 mb-6 text-center">
          System Overview
        </h1>
        <p className="text-gray-700 text-lg leading-relaxed">{summary}</p>
      </div>
    </main>
  );
}

