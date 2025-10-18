import React from "react";

export default function Navbar(){

  return(
    <>
        <nav className="w-full flex items-center justify-between py-4 px-8 bg-white shadow-md">
            <div className="text-2xl font-bold text-gray-600">Hacking The Climate</div>
            <div>
                <a href="/" className="text-gray-600 hover:text-gray-800 mx-4">Home</a>
                <a href="/about" className="text-gray-600 hover:text-gray-800 mx-4">About</a>
                <a href="/adminPanel" className="text-gray-600 hover:text-gray-800 mx-4">Admin Panel</a>
                <a href="/reviews" className="text-gray-600 hover:text-gray-800 mx-4">Reviews</a>
            </div>
        </nav>
    </>
  );
}