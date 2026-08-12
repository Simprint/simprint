import { createRoot } from 'react-dom/client';
import '../plugins/pages/splashscreen/src/styles/animations.css';
import './index.css';
import { SplashscreenPage } from '../plugins/pages/splashscreen/src/components/splashscreen-page';

createRoot(document.getElementById('root')!).render(<SplashscreenPage />);
