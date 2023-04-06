import { Route, Routes, MemoryRouter } from 'react-router-dom';
import Home from '../../modules/home';
import Settings from '../../modules/settings';

export default () => {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </MemoryRouter>
  );
};
