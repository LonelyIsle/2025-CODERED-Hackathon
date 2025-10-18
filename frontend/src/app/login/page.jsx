"use client";
import { motion } from "framer-motion";
import react from 'react';

export default function Login(){

    return(
        <>
            <main className="h-[calc(100vh-64px)] bg-gradient-to-b from-emerald-50 to-white flex items-center justify-center p-6 overflow-hidden">
                <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.6 }}
                    className="w-full max-w-md bg-white shadow-xl rounded-2xl p-8 border border-gray-100"
                >
                    <h1 className="text-3xl font-bold text-emerald-700 text-center mb-6">
                    Welcome Back
                    </h1>
                    <p className="text-gray-500 text-center mb-8">
                    Log in to access your AI-powered climate dashboard
                    </p>

                    <form className="space-y-5">
                    <div>
                        <label className="block text-sm font-medium text-gray-600 mb-1">
                        Username
                        </label>
                        <input
                        type="text"
                        placeholder="Enter your username"
                        className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-400"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-gray-600 mb-1">
                        Password
                        </label>
                        <input
                        type="password"
                        placeholder="Enter your password"
                        className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-400"
                        />
                    </div>

                    <button
                        type="submit"
                        className="w-full bg-emerald-600 text-white py-2 rounded-lg font-semibold hover:bg-emerald-700 transition duration-200 cursor-pointer"
                    >
                        Log In
                    </button>
                    </form>

                    <p className="text-center text-sm text-gray-500 mt-6">
                    Don’t have an account?{" "}
                    <a
                        href="#"
                        className="text-emerald-600 hover:text-emerald-800 font-medium"
                    >
                        Sign up
                    </a>
                    </p>
                </motion.div>
            </main>
        </>
    );
}