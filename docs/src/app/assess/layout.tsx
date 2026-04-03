import type { ReactNode } from 'react';

export const metadata = {
  title: 'Assessment Dashboard',
  description:
    'Continuous assessment dashboard for WeftOS projects. View code quality findings, project stats, and peer comparisons.',
};

export default function AssessLayout({ children }: { children: ReactNode }) {
  return <>{children}</>;
}
