"use client";

import { useState } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
} from "recharts";

export default function DashboardPage() {
  const [selectedCategory, setSelectedCategory] = useState("oil");
  const [selectedParams, setSelectedParams] = useState({
    emissions: true,
    efficiency: true,
  });

  // Mock API-like data
  const dataSets = {
    oil: [
      { year: 2018, emissions: 320, efficiency: 60 },
      { year: 2019, emissions: 340, efficiency: 63 },
      { year: 2020, emissions: 310, efficiency: 68 },
      { year: 2021, emissions: 355, efficiency: 70 },
      { year: 2022, emissions: 330, efficiency: 74 },
    ],
    electric: [
      { year: 2018, emissions: 150, efficiency: 70 },
      { year: 2019, emissions: 140, efficiency: 75 },
      { year: 2020, emissions: 130, efficiency: 80 },
      { year: 2021, emissions: 110, efficiency: 85 },
      { year: 2022, emissions: 100, efficiency: 89 },
    ],
    other: [
      { year: 2018, emissions: 210, efficiency: 65 },
      { year: 2019, emissions: 190, efficiency: 67 },
      { year: 2020, emissions: 185, efficiency: 69 },
      { year: 2021, emissions: 170, efficiency: 72 },
      { year: 2022, emissions: 160, efficiency: 75 },
    ],
  };

  const currentData = dataSets[selectedCategory];

  // Hardcoded recommendations
  const recommendations = [
    "Reduce CO₂ emissions by implementing cleaner energy sources.",
    "Increase energy efficiency through modern technology upgrades.",
    "Monitor and optimize industrial processes for lower environmental impact.",
    "Encourage adoption of renewable energy across sectors.",
  ];

  return (
    <main className="min-h-screen bg-gradient-to-b from-emerald-50 to-white p-8">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <h1 className="text-4xl font-bold text-emerald-700 mb-8 text-center">
          Environmental Impact Dashboard
        </h1>

        {/* Category Selector */}
        <div className="flex justify-center gap-4 mb-6 flex-wrap">
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="border border-emerald-300 text-emerald-800 rounded-lg p-3 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-emerald-500"
          >
            <option value="oil">Oil & Gas</option>
            <option value="electric">Electric</option>
            <option value="other">Other Services</option>
          </select>

          {/* Parameter checkboxes */}
          <label className="flex items-center gap-2 text-gray-700">
            <input
              type="checkbox"
              checked={selectedParams.emissions}
              onChange={() =>
                setSelectedParams((prev) => ({
                  ...prev,
                  emissions: !prev.emissions,
                }))
              }
              className="h-4 w-4 accent-emerald-500"
            />
            Show Emissions
          </label>
          <label className="flex items-center gap-2 text-gray-700">
            <input
              type="checkbox"
              checked={selectedParams.efficiency}
              onChange={() =>
                setSelectedParams((prev) => ({
                  ...prev,
                  efficiency: !prev.efficiency,
                }))
              }
              className="h-4 w-4 accent-emerald-500"
            />
            Show Efficiency
          </label>
        </div>

        {/* Graph */}
        <div className="bg-white rounded-2xl shadow-lg p-6 mb-8 border border-gray-100">
          <h2 className="text-xl font-semibold text-gray-700 mb-4">
            {selectedCategory === "oil"
              ? "Oil & Gas Emission Trends"
              : selectedCategory === "electric"
              ? "Electric Sector Emission Trends"
              : "Other Services Emission Trends"}
          </h2>
          <div className="h-80">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={currentData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="year" />
                <YAxis />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#fff",
                    color: "#000000",
                    borderRadius: "8px",
                    border: "none",
                  }}
                />
                {selectedParams.emissions && (
                  <Line
                    type="monotone"
                    dataKey="emissions"
                    stroke="#10b981"
                    strokeWidth={3}
                    dot={{ r: 4 }}
                  />
                )}
                {selectedParams.efficiency && (
                  <Line
                    type="monotone"
                    dataKey="efficiency"
                    stroke="#3b82f6"
                    strokeWidth={3}
                    dot={{ r: 4 }}
                  />
                )}
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Data Table */}
        <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100 mb-8">
          <h2 className="text-xl font-semibold text-gray-700 mb-4">
            Data Summary
          </h2>
          <table className="w-full border-collapse">
            <thead>
              <tr className="bg-emerald-100 text-emerald-800">
                <th className="p-3 text-left">Year</th>
                {selectedParams.emissions && (
                  <th className="p-3 text-left">CO₂ Emissions (tons)</th>
                )}
                {selectedParams.efficiency && (
                  <th className="p-3 text-left">Energy Efficiency (%)</th>
                )}
              </tr>
            </thead>
            <tbody>
              {currentData.map((d, i) => (
                <tr
                  key={i}
                  className={`border-t ${
                    i % 2 === 0 ? "bg-gray-50" : "bg-white"
                  } text-gray-700`}
                >
                  <td className="p-3">{d.year}</td>
                  {selectedParams.emissions && <td className="p-3">{d.emissions}</td>}
                  {selectedParams.efficiency && <td className="p-3">{d.efficiency}</td>}
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Recommendations */}
        <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
          <h2 className="text-xl font-semibold text-gray-700 mb-4">
            Mitigation Recommendations
          </h2>
          <ul className="list-disc list-inside space-y-2 text-gray-700">
            {recommendations.map((rec, i) => (
              <li key={i}>{rec}</li>
            ))}
          </ul>
        </div>
      </div>
    </main>
  );
}