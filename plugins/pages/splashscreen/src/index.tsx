import '../../../../src/index.css';
import './styles/animations.css';
import { SplashscreenPage } from './components/splashscreen-page';

const splashscreenPlugin = {
  id: 'splashscreen',
  name: 'Splashscreen',
  version: '1.0.0',
  component: SplashscreenPage,
  slots: [],
};

export default splashscreenPlugin;
