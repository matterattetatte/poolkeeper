// Import the functions you need from the SDKs you need
import { initializeApp } from "firebase/app";
import { getAnalytics } from "firebase/analytics";
// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
  apiKey: "AIzaSyB_RJrBEuEqzIMi6BrdaLcZFPMA4hDL9hk",
  authDomain: "poolkeeper-ad847.firebaseapp.com",
  projectId: "poolkeeper-ad847",
  storageBucket: "poolkeeper-ad847.firebasestorage.app",
  messagingSenderId: "22196868647",
  appId: "1:22196868647:web:407d72ebbcdde9d6d6e002",
  measurementId: "G-SS4VLXP5C7"
};

// Initialize Firebase
const firebaseApp = initializeApp(firebaseConfig);
export const analytics = getAnalytics(firebaseApp);


