"use client";
import { motion } from "framer-motion";

export default function ReviewsPage() {
  const reviews = [
    {
      name: "Dr. Elena Torres",
      role: "Environmental Scientist",
      review:
        "This platform revolutionizes the way we visualize climate data. The AI modeling is both accurate and intuitive — perfect for environmental research.",
      image: "https://randomuser.me/api/portraits/women/65.jpg",
    },
    {
      name: "Mark Reynolds",
      role: "Data Analyst",
      review:
        "The ability to generate predictive models in real-time is astounding. I’ve used many tools, but none blend usability and depth like this one.",
      image: "https://randomuser.me/api/portraits/men/12.jpg",
    },
    {
      name: "Aisha Chen",
      role: "Sustainability Officer",
      review:
        "This app made environmental reporting effortless for our organization. The insights we gained helped us create more effective sustainability strategies.",
      image: "https://randomuser.me/api/portraits/women/30.jpg",
    },
    {
      name: "Carlos Mendoza",
      role: "Climate Policy Advisor",
      review:
        "I love how the AI explains its predictions. It’s not a black box — it actually helps us communicate complex findings to non-technical stakeholders.",
      image: "https://randomuser.me/api/portraits/men/32.jpg",
    },
    {
      name: "Priya Singh",
      role: "University Researcher",
      review:
        "The integration of machine learning with real environmental data is simply brilliant. It’s a tool I now use in all my student projects.",
      image: "https://randomuser.me/api/portraits/women/58.jpg",
    },
  ];

  return (
    <main className="min-h-screen bg-gradient-to-b from-emerald-50 to-white p-8">
      <div className="max-w-6xl mx-auto text-center mb-12">
        <motion.h1
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="text-4xl font-bold text-emerald-700 mb-4"
        >
          What Experts Are Saying
        </motion.h1>
        <p className="text-gray-600 max-w-2xl mx-auto">
          Hear from professionals who are using our AI-powered climate modeling
          app to better understand and report on environmental changes.
        </p>
      </div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.3, duration: 0.8 }}
        className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-8"
      >
        {reviews.map((r, i) => (
          <motion.div
            key={i}
            whileHover={{ scale: 1.03 }}
            className="bg-white shadow-lg rounded-2xl p-6 border border-gray-100"
          >
            <div className="flex items-center gap-4 mb-4">
              <img
                src={r.image}
                alt={r.name}
                className="w-14 h-14 rounded-full object-cover border border-emerald-200"
              />
              <div>
                <h3 className="text-lg font-semibold text-emerald-800">
                  {r.name}
                </h3>
                <p className="text-sm text-gray-500">{r.role}</p>
              </div>
            </div>
            <p className="text-gray-700 italic">“{r.review}”</p>
          </motion.div>
        ))}
      </motion.div>
    </main>
  );
}

