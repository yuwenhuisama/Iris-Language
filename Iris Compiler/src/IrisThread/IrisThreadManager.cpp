#include "IrisThread/IrisThreadManager.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"

//static size_t s_nMainThreadID;
//
//thread::id IrisThreadManager::s_nMainThreadID;
//
//unordered_map<thread::id, IrisThreadUniqueInfo*> IrisThreadManager::s_mpThreadInfoMap;
//unordered_map<thread::id, thread*> IrisThreadManager::s_mpThreadMap;
//unordered_map<thread::id, bool> IrisThreadManager::s_mpThreadBlockedMap;
//
//recursive_mutex IrisThreadManager::s_rmNewThreadMT;
//recursive_mutex IrisThreadManager::s_rmNewThreadInfoMT;
//recursive_mutex IrisThreadManager::s_rmTransferMT;
//recursive_mutex IrisThreadManager::s_rmThreadSetMT;
//recursive_mutex IrisThreadManager::s_rmThreadQueryMT;

IrisThreadUniqueInfo * IrisDev_GetCurrentThreadInfo()
{
	return IrisThreadManager::CurrentThreadManager()->GetThreadInfo(this_thread::get_id());
}

bool IrisDev_CurrentThreadIsMainThread()
{
	return IrisThreadManager::CurrentThreadManager()->IsMainThread();
}

IrisThreadManager* IrisThreadManager::sm_pInstance = nullptr;

IrisThreadManager* IrisThreadManager::CurrentThreadManager() {
	if (!sm_pInstance) {
		sm_pInstance = new IrisThreadManager;
	}
	return sm_pInstance;
}

void IrisThreadManager::SetCurrentThreadManager(IrisThreadManager * pManager) {
	sm_pInstance = pManager;
}

void IrisThreadManager::Initialize()
{
	s_nMainThreadID = this_thread::get_id();
	AddNewThreadInfo(s_nMainThreadID, new IrisThreadUniqueInfo());
	s_mpThreadBlockedMap.insert(pair<thread::id, bool>(this_thread::get_id(), false));
}

void IrisThreadManager::AddNewThread(const thread::id& nThreadID, thread* pThread)
{
	lock_guard<recursive_mutex> lgLock(s_rmNewThreadMT);
	s_mpThreadMap.insert(pair<thread::id, thread*>(nThreadID, pThread));
	s_mpThreadBlockedMap.insert(pair<thread::id, bool>(nThreadID, false));
}

void IrisThreadManager::AddNewThreadInfo(const thread::id& nThreadID, IrisThreadUniqueInfo * pInfo)
{
	lock_guard<recursive_mutex> lgLock(s_rmNewThreadInfoMT);
	s_mpThreadInfoMap.insert(pair<thread::id, IrisThreadUniqueInfo*>(nThreadID, pInfo));
}

IrisThreadUniqueInfo * IrisThreadManager::GetThreadInfo(const thread::id& nThreadID)
{
	return s_mpThreadInfoMap[nThreadID];
}

void IrisThreadManager::DeleteThreadInfo(const thread::id& nThreadID)
{
	lock_guard<recursive_mutex> lgLock(s_rmNewThreadInfoMT);
	auto pData = s_mpThreadInfoMap[this_thread::get_id()];
	delete pData;
	s_mpThreadInfoMap.erase(nThreadID);
}

void IrisThreadManager::DeleteThread(const thread::id & nThreadID)
{
	lock_guard<recursive_mutex> lgLock(s_rmNewThreadInfoMT);
	s_mpThreadMap.erase(nThreadID);
	s_mpThreadBlockedMap.erase(nThreadID);
}

void IrisThreadManager::DetachAllThread()
{
	for (auto& pThread : s_mpThreadMap) {
		if (!pThread.second->joinable()) {
			pThread.second->detach();
			delete pThread.second;
		}
	}

	for (auto& pInfo : s_mpThreadInfoMap) {
		delete pInfo.second;
	}
}

bool IrisThreadManager::IsMainThread()
{
	return this_thread::get_id() == s_nMainThreadID;
}

void IrisThreadManager::TransferCurrentThreadHeapToMainThread()
{
	lock_guard<recursive_mutex> lgLock(s_rmTransferMT);
	auto pMainData = GetThreadInfo(s_nMainThreadID);
	auto pCurrentData = GetThreadInfo(this_thread::get_id());
	auto& stMainHeap = pMainData->m_ehEnvironmentHeap;
	auto& stCurrentHeap = pCurrentData->m_ehEnvironmentHeap;
	stMainHeap.insert(stCurrentHeap.begin(), stCurrentHeap.end());
}

void IrisThreadManager::SetThreadBlock(const thread::id& nThreadID, bool bBlocked)
{
	lock_guard<recursive_mutex> lgLock(s_rmThreadSetMT);
	s_mpThreadBlockedMap[nThreadID] = bBlocked;
}

bool IrisThreadManager::IsAllThreadBlocked()
{
	for (auto& pState : s_mpThreadBlockedMap) {
		if (!pState.second) {
			return false;
		}
	}
	return true;
}

void IrisThreadManager::ReleaseAllThreadData()
{
	for (auto& pData : s_mpThreadInfoMap) {
		delete pData.second;
	}
}

IrisThreadManager::IrisThreadManager()
{
}


IrisThreadManager::~IrisThreadManager()
{
}
