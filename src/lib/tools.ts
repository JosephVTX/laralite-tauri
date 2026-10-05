import type { ToolId } from "./api";

export const TOOLS: { id: ToolId; label: string; hint: string; path?: false }[] = [
  { id: "php", label: "PHP", hint: "windows.php.net · NTS recomendado para CLI/FastCGI, TS para Apache" },
  { id: "apache", label: "Apache", hint: "Apache Lounge · requiere VC++ Redistributable" },
  { id: "mysql", label: "MySQL", hint: "cdn.mysql.com" },
  { id: "mariadb", label: "MariaDB", hint: "downloads.mariadb.org" },
  { id: "phpmyadmin", label: "phpMyAdmin", hint: "phpmyadmin.net · ya viene configurado: entra directo sin usuario ni clave", path: false },
  { id: "postgres", label: "PostgreSQL", hint: "EDB · usuario postgres sin clave, puerto 5432" },
  { id: "heidisql", label: "HeidiSQL", hint: "heidisql.com · cliente gráfico con tus conexiones locales ya creadas", path: false },
  { id: "composer", label: "Composer", hint: "getcomposer.org · composer.phar + composer.bat" },
];

export const toolLabel = (id: ToolId) => TOOLS.find((t) => t.id === id)!.label;
