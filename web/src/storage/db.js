// web/src/storage/db.js
// High-Capacity IndexedDB Persistence Layer for XiangRust AI Engine
// Single-Word English Identifiers

const name = 'xiangrust';
const version = 1;

function open() {
  return new Promise((resolve, reject) => {
    if (typeof window === 'undefined' || !window.indexedDB) {
      resolve(null);
      return;
    }
    const request = window.indexedDB.open(name, version);
    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      if (!db.objectStoreNames.contains('experience')) {
        db.createObjectStore('experience', { keyPath: 'id', autoIncrement: true });
      }
      if (!db.objectStoreNames.contains('history')) {
        db.createObjectStore('history', { keyPath: 'id' });
      }
    };
    request.onsuccess = (event) => resolve(event.target.result);
    request.onerror = (event) => reject(event.target.error);
  });
}

export async function saveExperienceDb(sample) {
  try {
    const db = await open();
    if (!db) return false;
    return new Promise((resolve, reject) => {
      const tx = db.transaction('experience', 'readwrite');
      const store = tx.objectStore('experience');
      store.put({ ...sample, stamp: Date.now() });
      tx.oncomplete = () => resolve(true);
      tx.onerror = () => reject(tx.error);
    });
  } catch (err) {
    console.error('IndexedDB save error:', err);
    return false;
  }
}

export async function loadExperienceDb() {
  try {
    const db = await open();
    if (!db) return [];
    return new Promise((resolve, reject) => {
      const tx = db.transaction('experience', 'readonly');
      const store = tx.objectStore('experience');
      const request = store.getAll();
      request.onsuccess = () => resolve(request.result || []);
      request.onerror = () => reject(request.error);
    });
  } catch (err) {
    console.error('IndexedDB load error:', err);
    return [];
  }
}
