"use client";

import { FaGithub, FaLinkedin } from "react-icons/fa";
import { SiCashapp } from "react-icons/si";

export default function AboutPage() {
  const team = [
    {
      name: "Cole Plagens",
      role: "Frontend Developer",
      image: "/cole.png",
      github: "https://github.com/Colep39",
      linkedin: "#",
      cashapp: "$colep39",
    },
    {
      name: "Henry Moran",
      role: "Backend Developer",
      image: "/henry.jpg",
      github: "https://github.com/plobethus",
      linkedin: "#",
      cashapp: "#",
    },
    {
      name: "William Stewart",
      role: "AI Engineer",
      image: "/willyboy.png",
      github: "https://github.com/LonelyIsle",
      linkedin: "#",
      cashapp: "#",
    },
    {
      name: "Naomi Ayub",
      role: "Project Manager",
      image: "/naomi.jpg",
      github: "https://github.com/AyubNaomi",
      linkedin: "#",
      cashapp: "#",
    },
  ];

  return (
    <main className="min-h-screen bg-gradient-to-b from-emerald-50 to-white p-8">
      <div className="max-w-7xl mx-auto text-center">
        {/* Page Header */}
        <h1 className="text-4xl font-bold text-emerald-700 mb-4">
          Meet Our Team
        </h1>
        <p className="text-gray-600 mb-12 max-w-2xl mx-auto">
          We are a passionate group of developers and climate enthusiasts working on AI-powered tools for environmental insights. Connect with us below!
        </p>

        {/* Team Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-8">
          {team.map((member, i) => (
            <div
              key={i}
              className="bg-white rounded-2xl shadow-lg p-6 flex flex-col items-center hover:scale-105 transition-transform duration-300 border border-gray-100 relative"
            >
              <img
                src={member.image}
                alt={member.name}
                className="w-28 h-28 rounded-full object-cover mb-4 border-2 border-emerald-200"
              />
              <h3 className="text-xl font-semibold text-emerald-800">{member.name}</h3>
              <p className="text-gray-500 mb-4">{member.role}</p>

              {/* Social Links */}
              <div className="flex gap-4 items-center relative">
                <a
                  href={member.github}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-gray-600 hover:text-gray-800 text-2xl"
                >
                  <FaGithub />
                </a>
                <a
                  href={member.linkedin}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 hover:text-blue-800 text-2xl"
                >
                  <FaLinkedin />
                </a>

                {/* CashApp Tooltip */}
                {member.cashapp !== "#" && (
                  <div className="relative group">
                    <div className="text-green-500 text-2xl cursor-default">
                      <SiCashapp />
                    </div>
                    <span className="absolute bottom-full mb-2 px-2 py-1 text-sm text-white bg-green-600 rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap">
                      {member.cashapp}
                    </span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}
