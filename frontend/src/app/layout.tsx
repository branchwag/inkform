import type { Metadata } from "next";
import { ThemeToggle } from "../components/theme-toggle";
import "./globals.css";

export const metadata: Metadata = {
  title: "Inkform",
  description: "Turn guided handwriting samples into a personal digital font."
};

const themeBootstrap = `
try {
  const storedTheme = localStorage.getItem("inkform-theme");
  const theme = storedTheme === "dark" || storedTheme === "light"
    ? storedTheme
    : matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  document.documentElement.dataset.theme = theme;
} catch {}
`;

type RootLayoutProps = Readonly<{
  children: React.ReactNode;
}>;

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeBootstrap }} />
      </head>
      <body>
        <ThemeToggle />
        {children}
      </body>
    </html>
  );
}
