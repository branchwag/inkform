import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Inkform",
  description: "Turn guided handwriting samples into a personal digital font."
};

type RootLayoutProps = Readonly<{
  children: React.ReactNode;
}>;

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
