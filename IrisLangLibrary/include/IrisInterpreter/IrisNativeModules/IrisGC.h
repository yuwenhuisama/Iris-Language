#ifndef _H_IRISGC_
#define _H_IRISGC_

#include "IrisInterpreter/IrisStructure/IrisClass.h"
#include "IrisInterpreter/IrisStructure/IrisObject.h"
#include "IrisUnil/IrisTree.h"
#include <thread>
#include <mutex>
#include <unordered_map>
using namespace std;

class IrisGC
{
public:
	struct ThreadGCData {
		//size_t m_nCurrentHeapSize = 0;
		//size_t m_nNextThresholdSize = IrisGC::c_nThreshold;

		size_t m_nCurrentContextEnvironmentHeapSize = 0;
		size_t m_nNextContextEnvironmentThresholdSize = c_nThreshold;
	};

private:
	static const int c_nThreshold = 1024;
	static IrisGC* sm_pInstance;

private:
	unordered_map<thread::id, ThreadGCData*> sm_mpThreadGCDataMap;
	ThreadGCData* sm_pMainThreadGCData = nullptr;

	int sm_nCurrentHeapSize = 0;
	int sm_nNextThresholdSize = c_nThreshold;

	//static int sm_nCurrentContextEnvrionmentHeapSize;
	//static int sm_nNextContextEnvrionmentThresholdSize;

	bool sm_bFlag = true;
	recursive_mutex sm_rmGCMutex;
	recursive_mutex sm_rmNewDataMutex;
	recursive_mutex sm_rmTransferDataMutex;

public:
	mutex sm_mtStopTheWorldFinishedMutex;
	condition_variable sm_cvGCStopTheWorldConditionVariable;
	condition_variable sm_cvGCStopTheWorldFinishedConditionVariable;
	bool sm_bStopTheWorld = false;

	condition_variable sm_cvGCWakeUpConditionVariable;
	mutex sm_mtGCWakeUpMutex;
	bool sm_bWakeUpGCThread = false;
	bool sm_bGCThreadShutUp = false;

	thread* sm_pGCThread = nullptr;
	bool sm_bGCWaitOver = false;

private:

	static void _GCThreadFunction();

	void _ErgodicTreeAndMark(IrisModule* pCurNode);

	void _ClearMark();
	void _Mark();
	void _Sweep();

public:
	static IrisGC* CurrentGC();
	static void SetCurrentGC(IrisGC* pGC);

public:
	void AddThreadGCData(const thread::id& nID, ThreadGCData* pThreadGCData);
	void RemoveThreadGCData(const thread::id& nId);
	ThreadGCData* GetCurrentThreadGCData();

	void ResetNextThreshold();
	void Start();
	void ForceStart();
	void SetGCFlag(bool bFlag);
	void AddSize(int nSize);
	
	void AddContextEnvironmentSize();
	void ContextEnvironmentGC();

	void TransferCurrentGCDataToMainThread();

	void Initialize();
	void ReleaseAllThreadData();

private:
	IrisGC();

public:
	~IrisGC();
};

#endif