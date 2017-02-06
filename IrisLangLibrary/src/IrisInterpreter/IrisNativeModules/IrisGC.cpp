#include "IrisInterpreter/IrisNativeModules/IrisGC.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeClasses/IrisMethodBase.h"
#include "IrisInterpreter/IrisStructure/IrisClosureBlock.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"
#include "IrisUnil/IrisTree.h"
#include <unordered_set>
using namespace std;

IrisGC* IrisGC::sm_pInstance = nullptr;

IrisGC* IrisGC::CurrentGC() {
	if (!sm_pInstance) {
		sm_pInstance = new IrisGC;
	}
	return sm_pInstance;
}

void IrisGC::SetCurrentGC(IrisGC * pGC) {
	sm_pInstance = pGC;
}

void IrisGC::_ErgodicTreeAndMark(IrisModule* pCurNode) {
	static_cast<IrisObject*>(pCurNode->m_pModuleObject)->Mark();
	for (auto& value : pCurNode->m_hsClassVariables) {
		static_cast<IrisObject*>(value.second.GetIrisObject())->Mark();
	}
	for (auto& value : pCurNode->m_hsConstances) {
		static_cast<IrisObject*>(value.second.GetIrisObject())->Mark();
	}
	for (auto& value : pCurNode->m_hsInstanceMethods) {
		static_cast<IrisObject*>(value.second->m_pMethodObject)->Mark();
	}
	for (auto& value : pCurNode->m_hsClasses) {
		static_cast<IrisObject*>((value->m_pClassObject))->Mark();
		for (auto& value2 : value->m_hsClassVariables) {
			static_cast<IrisObject*>((value2.second.GetIrisObject()))->Mark();
		}
		for (auto& value2 : value->m_hsConstances) {
			static_cast<IrisObject*>((value2.second.GetIrisObject()))->Mark();
		}
		for (auto& value2 : value->m_hsInstanceMethods) {
			static_cast<IrisObject*>((value2.second->m_pMethodObject))->Mark();
		}
	}
	for (auto& value : pCurNode->m_hsInvolvedInterfaces) {
		static_cast<IrisObject*>((value->m_pInterfaceObject))->Mark();
	}
	for (auto& value : pCurNode->m_hsInvolveModules) {
		_ErgodicTreeAndMark(value.second);
	}
}

void IrisGC::_ClearMark() {
	//IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	IrisHeap& ihHeap = IrisInterpreter::CurrentInterpreter()->m_hpHeap;
	unordered_set<IrisObject*>& stObjects = ihHeap.GetHeapSet();
	for (unordered_set<IrisObject*>::iterator iter = stObjects.begin(); iter != stObjects.end(); ) {
		(*iter)->ClearMark();
		++iter;
	}
}

void IrisGC::_Mark() {
	// 需要Mark的对象：临时栈、全局变量、Main环境下的变量/常量、所有类/模块的类变量/常量、所有可引用到的对象的实例变量、以及所有可以引用到的Method对象。
	static_cast<IrisObject*>(((IrisInterpreter::CurrentInterpreter()->Nil()).GetIrisObject()))->Mark();
	static_cast<IrisObject*>(((IrisInterpreter::CurrentInterpreter()->True()).GetIrisObject()))->Mark();
	static_cast<IrisObject*>(((IrisInterpreter::CurrentInterpreter()->False()).GetIrisObject()))->Mark();
	auto pInter = IrisInterpreter::CurrentInterpreter();

	for (auto& pThreadInfoIter : IrisThreadManager::CurrentThreadManager()->s_mpThreadInfoMap) {
		auto pThreadInfo = pThreadInfoIter.second;

		// 临时新生对象
		for (auto& pObject : pThreadInfo->m_skTempNewObjectStack) {
			static_cast<IrisObject*>((pObject))->Mark();
		}

		// Counter栈
		for (auto& value : pThreadInfo->m_skCounterRegister) {
			if (value.GetIrisObject()) {
				static_cast<IrisObject*>((value.GetIrisObject()))->Mark();
			}
		}

		// Timer栈
		for (auto& value : pThreadInfo->m_skTimerRegister) {
			if (value.GetIrisObject()) {
				static_cast<IrisObject*>((value.GetIrisObject()))->Mark();
			}
		}

		// Vessle栈
		for (auto& value : pThreadInfo->m_skVessleRegister) {
			if (value.GetIrisObject()) {
				static_cast<IrisObject*>((value.GetIrisObject()))->Mark();
			}
		}

		// Iterator栈
		for (auto& value : pThreadInfo->m_skIteratorRegister) {
			if (value.GetIrisObject()) {
				static_cast<IrisObject*>((value.GetIrisObject()))->Mark();
			}
		}

		// 寄存器组
		if (pThreadInfo->m_ivResultRegister.GetIrisObject()) {
			static_cast<IrisObject*>((pThreadInfo->m_ivResultRegister.GetIrisObject()))->Mark();
		}
		if (pThreadInfo->m_ivCounterRegister.GetIrisObject()) {
			static_cast<IrisObject*>((pThreadInfo->m_ivCounterRegister.GetIrisObject()))->Mark();
		}
		if (pThreadInfo->m_ivTimerRegister.GetIrisObject()) {
			static_cast<IrisObject*>((pThreadInfo->m_ivTimerRegister.GetIrisObject()))->Mark();
		}
		if (pThreadInfo->m_ivVessleRegister.GetIrisObject()) {
			static_cast<IrisObject*>((pThreadInfo->m_ivVessleRegister.GetIrisObject()))->Mark();
		}
		if (pThreadInfo->m_ivIteratorRegister.GetIrisObject()) {
			static_cast<IrisObject*>((pThreadInfo->m_ivIteratorRegister.GetIrisObject()))->Mark();
		}
		if (pThreadInfo->m_ivCompareRegister.GetIrisObject()) {
			static_cast<IrisObject*>((pThreadInfo->m_ivCompareRegister.GetIrisObject()))->Mark();
		}
		if (pThreadInfo->m_bIrregularHappenedRegister) {
			static_cast<IrisObject*>((pThreadInfo->m_ivIrregularObjectRegister.GetIrisObject()))->Mark();
		}

		// 临时栈
		for (auto& value : pThreadInfo->m_stStack.m_lsStack) {
			static_cast<IrisObject*>((value.GetIrisObject()))->Mark();
		}

		// Native临时栈
		for (auto& value : pThreadInfo->m_hpObjectInNativeFunctionHeap) {
			value->Mark();
		}
	}
	
	//if (IrisDevUtil::CurrentThreadIsMainThread()) {
	// 全局变量
	for (auto& value : pInter->m_mpGlobalValues) {
		static_cast<IrisObject*>((value.second.GetIrisObject()))->Mark();
	}
	// Main环境变量
	for (auto& value : pInter->m_mpOtherValues) {
			static_cast<IrisObject*>((value.second.GetIrisObject()))->Mark();
	}
	// Main环境常量
	for (auto& value : pInter->m_mpConstances) {
		if (IrisDevUtil::CheckClassIsModule(value.second)) {
			IrisModule* pModule = IrisDevUtil::GetNativeModule(value.second)->GetInternModule();
			_ErgodicTreeAndMark(pModule);
		}
		// 开始遍历所有模块/类
		else if (IrisDevUtil::CheckClassIsClass(value.second)) {
			IrisClass* pClass = IrisDevUtil::GetNativeClass(value.second)->GetInternClass();
			static_cast<IrisObject*>((pClass->GetClassObject()))->Mark();
			for (auto value2 : pClass->m_hsClassVariables) {
				static_cast<IrisObject*>((value2.second.GetIrisObject()))->Mark();
			}
			for (auto value2 : pClass->m_hsConstances) {
				static_cast<IrisObject*>((value2.second.GetIrisObject()))->Mark();
			}
			for (auto value2 : pClass->m_hsInstanceMethods) {
				static_cast<IrisObject*>((value2.second->m_pMethodObject))->Mark();
			}
		}
		else {
			static_cast<IrisObject*>((value.second.GetIrisObject()))->Mark();
		}
	}
	// Main环境方法
	for (auto& value : pInter->m_mpMethods) {
		static_cast<IrisObject*>((value.second->m_pMethodObject))->Mark();
	}
	for (auto& pThreadInfoIter : IrisThreadManager::CurrentThreadManager()->s_mpThreadInfoMap) {
		auto pThreadInfo = pThreadInfoIter.second;
		for (auto& env : pThreadInfo->m_skEnvironmentStack) {
			if (env) {
				++env->m_nReferenced;
			}
		}
		if (pThreadInfo->m_pEnvrionmentRegister) {
			++pThreadInfo->m_pEnvrionmentRegister->m_nReferenced;
		}

		// Environment
		unordered_set<IrisContextEnvironment*>& stEnvironments = pThreadInfo->m_ehEnvironmentHeap;
		auto pData = sm_mpThreadGCDataMap[pThreadInfoIter.first];
		for (unordered_set<IrisContextEnvironment*>::iterator iter = stEnvironments.begin(); iter != stEnvironments.end(); )
		{
			if ((*iter)->m_nReferenced == 0) {
				pData->m_nCurrentContextEnvironmentHeapSize -= sizeof(IrisContextEnvironment);
				delete (*iter);
				stEnvironments.erase(iter++);
			}
			else {
				++iter;
			}
		}

		// 环境
		for (auto& env : pThreadInfo->m_ehEnvironmentHeap) {
			for (auto value : env->m_mpVariables) {
				static_cast<IrisObject*>((value.second.GetIrisObject()))->Mark();
			}
		}

		for (auto& env : pThreadInfo->m_skEnvironmentStack) {
			if (env) {
				--env->m_nReferenced;
			}
		}

		if (pThreadInfo->m_pEnvrionmentRegister) {
			if (pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock) {
				static_cast<IrisObject*>(pThreadInfo->m_pEnvrionmentRegister->m_pClosureBlock->GetNativeObject())->Mark();
			}
		}

		if (pThreadInfo->m_pEnvrionmentRegister) {
			--pThreadInfo->m_pEnvrionmentRegister->m_nReferenced;
		}
	}

}

void IrisGC::_Sweep() {
	// 清扫
	IrisInterpreter* pInterpreter = IrisInterpreter::CurrentInterpreter();
	IrisHeap& ihHeap = pInterpreter->m_hpHeap;
	unordered_set<IrisObject*>& stObjects = ihHeap.GetHeapSet();
	for (unordered_set<IrisObject*>::iterator iter = stObjects.begin(); iter != stObjects.end(); )
	{
		if (!(*iter)->m_bIsMaked && !(*iter)->IsPermanent()) {
			if ((*iter)->m_bLiteralObject && !(*iter)->m_bLiteralObjectAssigned) {
				++iter;
			}
			else {
				if ((*iter)->GetClass()->GetInternClass()->GetClassName() != "Object") {
					sm_nCurrentHeapSize -= (*iter)->GetClass()->GetTrustteeSize((*iter)->GetNativeObject()) + sizeof(IrisObject);
				}
				else {
					sm_nCurrentHeapSize -= sizeof(IrisObject);
				}
				delete (*iter);
				stObjects.erase(iter++);
			}
		}
		else {
			++iter;
		}
	}
}

void IrisGC::AddThreadGCData(const thread::id & nID, ThreadGCData * pThreadGCData)
{
	lock_guard<recursive_mutex> lgLock(sm_rmGCMutex);
	sm_mpThreadGCDataMap.insert(pair<thread::id, ThreadGCData*>(nID, pThreadGCData));
}

void IrisGC::RemoveThreadGCData(const thread::id & nId)
{
	lock_guard<recursive_mutex> lgLock(sm_rmGCMutex);
	delete sm_mpThreadGCDataMap[this_thread::get_id()];
	sm_mpThreadGCDataMap.erase(this_thread::get_id());
}

IrisGC::ThreadGCData* IrisGC::GetCurrentThreadGCData()
{
	return sm_mpThreadGCDataMap[this_thread::get_id()];
}

void IrisGC::ResetNextThreshold() {
	sm_nNextThresholdSize = sm_nCurrentHeapSize + c_nThreshold;
}

void IrisGC::AddSize(int nSize) {
	sm_nCurrentHeapSize += nSize;
}

void IrisGC::SetGCFlag(bool bFlag) {
	sm_bFlag = bFlag;
}

void IrisGC::Start() {
	if (sm_bWakeUpGCThread) {
		return;
	}
	if(sm_bFlag) {
		unique_lock<mutex> ulLock(sm_mtGCWakeUpMutex);
		auto pData = GetCurrentThreadGCData();
		lock_guard<recursive_mutex> lgLock(sm_rmGCMutex);
		if(sm_nCurrentHeapSize > sm_nNextThresholdSize) {
			// Wake Up
			sm_bWakeUpGCThread = true;
			sm_cvGCWakeUpConditionVariable.notify_all();
		}
	}
}

void IrisGC::ForceStart() {
	if (sm_bWakeUpGCThread) {
		return;
	}
	if (sm_bFlag) {
		unique_lock<mutex> ulLock(sm_mtGCWakeUpMutex);
		sm_bWakeUpGCThread = true;
		sm_cvGCWakeUpConditionVariable.notify_all();
		//lock_guard<recursive_mutex> lgLock(sm_rmGCMutex);
		//auto pData = GetCurrentThreadGCData();
		//_ClearMark();
		//_Mark();
		//_Sweep();
		//if (sm_nCurrentHeapSize > sm_nNextThresholdSize) {
		//	sm_nNextThresholdSize = sm_nCurrentHeapSize + c_nThreshold;
		//}
	}
}

void IrisGC::AddContextEnvironmentSize() {
	GetCurrentThreadGCData()->m_nCurrentContextEnvironmentHeapSize += sizeof(IrisContextEnvironment);
}

void IrisGC::ContextEnvironmentGC() {
	if (sm_bFlag) {
		auto pData = GetCurrentThreadGCData();
		auto pThreadInfo = IrisDevUtil::GetCurrentThreadInfo();
		if (pData->m_nCurrentContextEnvironmentHeapSize > pData->m_nNextContextEnvironmentThresholdSize) {
			// Environment
			unordered_set<IrisContextEnvironment*>& stEnvironments = pThreadInfo->m_ehEnvironmentHeap;
			for (unordered_set<IrisContextEnvironment*>::iterator iter = stEnvironments.begin(); iter != stEnvironments.end(); )
			{
				if ((*iter)->m_nReferenced == 0) {
					pData->m_nCurrentContextEnvironmentHeapSize -= sizeof(IrisContextEnvironment);
					delete (*iter);
					stEnvironments.erase(iter++);
				}
				else {
					++iter;
				}
			}

			if (pData->m_nCurrentContextEnvironmentHeapSize > pData->m_nNextContextEnvironmentThresholdSize) {
				pData->m_nNextContextEnvironmentThresholdSize = pData->m_nCurrentContextEnvironmentHeapSize + c_nThreshold;
			}
		}
	}
}

void IrisGC::TransferCurrentGCDataToMainThread()
{
	lock_guard<recursive_mutex> lgLock(sm_rmTransferDataMutex);
	auto pCurrentData = sm_mpThreadGCDataMap[this_thread::get_id()];
	sm_pMainThreadGCData->m_nCurrentContextEnvironmentHeapSize += pCurrentData->m_nCurrentContextEnvironmentHeapSize;
}

void IrisGC::_GCThreadFunction() {
	auto pGC = IrisGC::CurrentGC();
	while (!pGC->sm_bGCThreadShutUp) {
		unique_lock<mutex> ulLock(pGC->sm_mtGCWakeUpMutex);
		while (!pGC->sm_bWakeUpGCThread) {
			pGC->sm_cvGCWakeUpConditionVariable.wait(ulLock);
			if (pGC->sm_bGCThreadShutUp) {
				break;
			}
		}
		unique_lock<mutex> ulStopLock(pGC->sm_mtStopTheWorldFinishedMutex);
		// Stop The World
		pGC->sm_bStopTheWorld = true;
		while (!IrisThreadManager::CurrentThreadManager()->IsAllThreadBlocked()) {
			pGC->sm_cvGCStopTheWorldFinishedConditionVariable.wait(ulStopLock);
		}

		pGC->_ClearMark();
		pGC->_Mark();
		pGC->_Sweep();

		if (pGC->sm_nCurrentHeapSize > pGC->sm_nNextThresholdSize) {
			pGC->sm_nNextThresholdSize = pGC->sm_nCurrentHeapSize + c_nThreshold;
		}

		//// Restart The World
		pGC->sm_bStopTheWorld = false;
		pGC->sm_bWakeUpGCThread = false;
		pGC->sm_cvGCStopTheWorldConditionVariable.notify_all();
	}
}

void IrisGC::Initialize()
{
	sm_pMainThreadGCData = new ThreadGCData();
	AddThreadGCData(this_thread::get_id(), sm_pMainThreadGCData);
	sm_pGCThread = new thread(_GCThreadFunction);
}

void IrisGC::ReleaseAllThreadData()
{
	for (auto& pData : sm_mpThreadGCDataMap) {
		delete pData.second;
	}
	sm_pGCThread->detach();
	sm_bGCThreadShutUp = true;
	delete sm_pGCThread;
}

IrisGC::IrisGC()
{
}


IrisGC::~IrisGC()
{
}
