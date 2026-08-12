import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App';
import { mainStartupTiming } from './main-startup-timing';

mainStartupTiming.moduleExecutedAt = performance.now();
createRoot(document.getElementById('root')!).render(<App mode="main" />);
mainStartupTiming.reactRenderScheduledAt = performance.now();

// createRoot(document.getElementById('root')!).render(
//   <StrictMode>
//     <App mode="main" />
//   </StrictMode>
// );
