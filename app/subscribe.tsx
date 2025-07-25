"use client";
import { useRouter } from "next/navigation";

export default function SubscribeButton() {
  const router = useRouter();

  const handleSubscribe = async () => {
    const res = await fetch("/api/stripe/checkout");
    const data = await res.json();
    router.push(data.url);
  };

  return (
    <button onClick={handleSubscribe}>
      Subscribe
    </button>
  );
}

