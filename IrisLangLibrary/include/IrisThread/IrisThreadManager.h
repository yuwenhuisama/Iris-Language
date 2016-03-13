#ifndef _H_IRISTHREADMANAGER_
#define _H_IRISTHREADMANAGER_
#include "IrisThreadUtil.h"
#include <unordered_map>
#include <thread>
#include <mutex>
#include <list>

using namespace std;
class IrisThreadManager
{
private:
	thread::id s_nMainThreadID;

	unordered_map<thread::id, IrisThreadUniqueInfo*> s_mpThreadInfoMap;
	unordered_map<thread::id, thread*> s_mpThreadMap;
	unordered_map<thread::id, bool> s_mpThreadBlockedMap;

	recursive_mutex s_rmNewThreadMT;
	recursive_mutex s_rmNewThreadInfoMT;
	recursive_mutex s_rmTransferMT;
	recursive_mutex s_rmThreadSetMT;
	recursive_mutex s_rmThreadQueryMT;

private:
	static IrisThreadManager* sm_pInstance;

public:
	static IrisThreadManager* CurrentThreadManager();
	static void SetCurrentThreadManager(IrisThreadManager* pManager);

public:

	void Initialize();
	void AddNewThread(const thread::id& nThreadID, thread* pThread);
	void AddNewThreadInfo(const thread::id& nThreadID, IrisThreadUniqueInfo* pInfo);
	IrisThreadUniqueInfo* GetThreadInfo(const thread::id& nThreadID);
	void DeleteThreadInfo(const thread::id& nThreadID);
	void DeleteThread(const thread::id& nThreadID);
	void DetachAllThread();
	bool IsMainThread();
	void TransferCurrentThreadHeapToMainThread();
	void SetThreadBlock(const thread::id& nThreadID, bool bBlocked);
	bool IsAllThreadBlocked();
	void ReleaseAllThreadData();

private:
	IrisThreadManager();

public:
	virtual ~IrisThreadManager();

	friend class IrisGC;
};

#endif