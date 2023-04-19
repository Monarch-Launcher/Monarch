import { Route, Routes, MemoryRouter } from 'react-router-dom';
import Home from '../../modules/home';
import Settings from '../../modules/settings';
import Library from '../../modules/library';
import Search from '../../modules/search';
import MooseTestPage from '../../modules/mooseTestPage';

export default () => {
  return (
    <MemoryRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/library" element={<Library />} />
        <Route path="/search" element={<Search />} />
        <Route path="/moose" element={<MooseTestPage />} />
      </Routes>
    </MemoryRouter>
  );
};