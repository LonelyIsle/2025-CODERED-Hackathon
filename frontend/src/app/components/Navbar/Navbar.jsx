import React from "react";
import Link from "next/link";

export default function Navbar() {
  return (
    <nav className="w-full flex items-center justify-between py-4 px-8 bg-white shadow-md">
      <div className="text-2xl font-bold text-gray-600">Hacking The Climate</div>
      <div>
        <Link href="/" className="text-gray-600 hover:text-gray-800 mx-4">Dashboard</Link>
        <Link href="/about" className="text-gray-600 hover:text-gray-800 mx-4">About</Link>
        <Link href="/adminPanel" className="text-gray-600 hover:text-gray-800 mx-4">Admin Panel</Link>
        <Link href="/reviews" className="text-gray-600 hover:text-gray-800 mx-4">Reviews</Link>
        <Link href="/logistics" className="text-gray-600 hover:text-gray-800 mx-4">Logistics</Link>
        <Link href="/login" className="text-gray-600 hover:text-gray-800 mx-4">Login</Link>
      </div>
    </nav>
  );
}
