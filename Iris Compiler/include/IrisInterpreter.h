#ifndef _H_IRISINTERPRETER_
#define _H_IRISINTERPRETER_

#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisUnil/IrisValue.h"
#include "IrisUnil/IrisTree.h"
#include "IrisUnil/IrisHeap.h"
#include "IrisUnil/IrisStack.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisDevelopUtil.h"
#include "IrisUnil/IrisCodeSegment.h"
#include "IrisThread/IrisWLLock.h"
#include <unordered_map>
#include <vector>
#include <list>
#include <unordered_set>
#include <thread>

#ifdef _MSC_VER
#include <Windows.h>
#endif

using namespace std;

class IrisObject;
class IrisInterface;
class IrisClass;
class IrisModule;
class IrisMethod;
class IrisContextEnvironment;
class IrisCompiler;
class IrisClosureBlock;

class IrisInterpreter
{
private:
	typedef unordered_map<string, IrisClass*> _ClassMap;
	typedef pair<string, IrisClass*> _ClassPair;
	typedef unordered_map<string, IrisInterface*> _InterfaceMap;
	typedef pair<string, IrisInterface*> _InterfacePair;
	typedef unordered_map<string, IrisValue> _ValueMap;
	typedef pair<string, IrisValue> _ValuePair;
	typedef unordered_map<string, IrisMethod*> _MethodMap;
	typedef pair<string, IrisMethod*> _MethodPair;
	typedef unordered_map<string, HMODULE> _ExtentionMap;
	typedef pair<string, HMODULE> _ExtentionPair;

	//typedef unordered_set<IrisContextEnvironment*> _EnvironmentHeap;
	//typedef vector<IrisContextEnvironment*> _EnvironmentStack;
	//typedef vector<unsigned int> _LoopDeepStack;
	//typedef vector<IrisValue> _ValueStack;
	//typedef vector<bool> _BooleanStack;

	typedef bool (*ExitConditonFunction)();

private:
	static IrisInterpreter* s_pInstance;

private:

	recursive_mutex m_rmHeapInsertMT;

	IrisHeap m_hpHeap;
	//IrisStack m_stStack;

	IrisTree<IrisModule*> m_trModuels;
	_ClassMap m_mpClasses;
	_InterfaceMap m_mpInterfaces;
	_MethodMap m_mpMethods;

	_ValueMap m_mpGlobalValues;
	_ValueMap m_mpOtherValues;
	_ValueMap m_mpConstances;

	IrisValue m_ivNil;
	IrisValue m_ivTrue;
	IrisValue m_ivFalse;

	IrisCompiler* m_pCurrentCompiler = nullptr;

	ExitConditonFunction m_pfExitConditionFunction = nullptr;

	IrisWLLock m_iwlConstanceLock;
	IrisWLLock m_iwlGlobalVariableLock;
	IrisWLLock m_iwlOtherVariableLock;
	IrisWLLock m_iwlMethodLock;

	IrisWLLock m_iwlInterfaceLock;
	IrisWLLock m_iwlClassLock;
	IrisWLLock m_iwlModuleLock;

	_ExtentionMap m_emLoadedExtention;

private:
	IrisInterpreter();
	int _Split(const string& str, list<string>& ret_, string sep = ",");

public:
	static IrisInterpreter* CurrentInterpreter();
	static void SetCurrentInterpreter(IrisInterpreter* pInter);

public:

	IrisAM GetOneAM(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);

	IrisClass* GetIrisClass(const string& strClassFullFieldName);
	IrisModule* GetIrisModule(const string& strModuleFullFiledName);
	IrisInterface* GetIrisInterface(const string& strInterfaceFullFiledName);

	IrisClass* GetIrisClass(const list<string>& lsRoute);
	IrisModule* GetIrisModule(const list<string>& lsRoute);
	IrisInterface* GetIrisInterface(const list<string>& lsRoute);

	bool RegistClass(const string& strClassFullFieldName, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(const string& strModuleFullFieldName, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(const string& strInterfaceFullFieldName, IIrisInterface* pInterface, bool bNative = true);

	bool RegistClass(list<string>& lsPath, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(list<string>& lsPath, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(list<string>& lsPath, IIrisInterface* pInterface, bool bNative = true);

	bool AddNewInstanceToHeap(IrisValue& ivValue);
	bool AddNewEnvironmentToHeap(IrisContextEnvironment* pEnvironment);

	bool Initialize();
	bool ShutDown();

public:

	inline void SetCompiler(IrisCompiler* pCompiler) { m_pCurrentCompiler = pCompiler; };
	bool Run();
	//bool RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer);
	bool RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer, unsigned int nCurrentFileIndex);
	inline void SetExitConditionFunction(ExitConditonFunction pfFunction) { m_pfExitConditionFunction = pfFunction; }

	bool LoadExtension(const string& strExtensionPath);

	~IrisInterpreter();

	// inlines
public:


	//inline void RegisiterClosuerBlock(IrisClosureBlock* pBlock) {
	//	m_pClosureBlockRegister = pBlock;
	//}

	void PopStack(unsigned int nTimes);

	inline bool FatalErrorHappened() { return IrisDev_GetCurrentThreadInfo()->m_bFatalErrorHappendRegister; }
	inline void HappenFatalError() { IrisDev_GetCurrentThreadInfo()->m_bFatalErrorHappendRegister = true; }

	inline bool IrregularHappened() { return IrisDev_GetCurrentThreadInfo()->m_bIrregularHappenedRegister; }
	inline void RegistIrregular(IrisValue& ivValue) {
		IrisDev_GetCurrentThreadInfo()->m_ivIrregularObjectRegister = ivValue; 
		IrisDev_GetCurrentThreadInfo()->m_bIrregularHappenedRegister = true;
	}
	inline void UnregistIrregular() { IrisDev_GetCurrentThreadInfo()->m_bIrregularHappenedRegister = false; }

	inline bool ForceStop() { return IrisDev_GetCurrentThreadInfo()->m_bIrregularHappenedRegister; }

	inline bool PushEnvironment() {
		IrisDev_GetCurrentThreadInfo()->m_skEnvironmentStack.push_back(IrisDev_GetCurrentThreadInfo()->m_pEnvrionmentRegister);
		return true;
	}
	inline bool SetEnvironment(IrisContextEnvironment* pEnvironment) {
		IrisDev_GetCurrentThreadInfo()->m_pEnvrionmentRegister = pEnvironment;
		return true;
	}
	inline bool PopEnvironment() {
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_pEnvrionmentRegister = pInfo->m_skEnvironmentStack.back();
		pInfo->m_skEnvironmentStack.pop_back();
		return true;
	}

	inline void PushVessle() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_skVessleRegister.push_back(pInfo->m_ivVessleRegister); 
	}
	inline void PopVessle() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_ivVessleRegister = pInfo->m_skVessleRegister.back();
		pInfo->m_skVessleRegister.pop_back(); 
	}

	inline void PushIterator() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_skIteratorRegister.push_back(pInfo->m_ivIteratorRegister); }
	inline void PopIterator() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_ivIteratorRegister = pInfo->m_skIteratorRegister.back();
		pInfo->m_skIteratorRegister.pop_back();
	}

	inline void PushCounter() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_skCounterRegister.push_back(pInfo->m_ivCounterRegister);
	}
	inline void PopCounter() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_ivCounterRegister = pInfo->m_skCounterRegister.back(); 
		pInfo->m_skCounterRegister.pop_back(); 
	}

	inline void PushTimer() {
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_skTimerRegister.push_back(pInfo->m_ivTimerRegister); }
	inline void PopTimer() {
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_ivTimerRegister = pInfo->m_skTimerRegister.back();
		pInfo->m_skTimerRegister.pop_back();
	}

	inline void PushUnlimitedLoopFlag() {
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_skUnimitedLoopFlag.push_back(pInfo->m_bUnimitedLoopFlagRegister); }
	inline void PopUnlimitedLoopFlag() { 
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		pInfo->m_bUnimitedLoopFlagRegister = pInfo->m_skUnimitedLoopFlag.back();
		pInfo->m_skUnimitedLoopFlag.pop_back(); 
	}

	inline unsigned int GetTopDeepIndex() {
		auto pInfo = IrisDev_GetCurrentThreadInfo();
		return pInfo->m_skDeepStack.empty() ? -1 : pInfo->m_skDeepStack.back(); 
	}
	inline void PushDeepIndex(unsigned int nIndex) { IrisDev_GetCurrentThreadInfo()->m_skDeepStack.push_back(nIndex); }
	inline void PopTopDeepIndex() { IrisDev_GetCurrentThreadInfo()->m_skDeepStack.pop_back(); }
	inline void ClearDeepStack() { IrisDev_GetCurrentThreadInfo()->m_skDeepStack.clear(); }

	inline unsigned int GetTopMethodDeepIndex() { return IrisDev_GetCurrentThreadInfo()->m_skMethodDeepStack.back(); }
	inline void PushMethodDeepIndex(unsigned int nIndex) { IrisDev_GetCurrentThreadInfo()->m_skMethodDeepStack.push_back(nIndex); }
	inline void PopMethodTopDeepIndex() { IrisDev_GetCurrentThreadInfo()->m_skMethodDeepStack.pop_back(); }
	inline void ClearMethodDeepStack() { IrisDev_GetCurrentThreadInfo()->m_skMethodDeepStack.clear(); }

	inline const IrisValue& GetCurrentResultRegister() { return IrisDev_GetCurrentThreadInfo()->m_ivResultRegister; }

	inline IrisContextEnvironment* GetCurrentContextEnvrionment() {
		return IrisDev_GetCurrentThreadInfo()->m_pEnvrionmentRegister;
	}

	inline void AddConstance(const string& strValueName, const IrisValue& ivValue) {
		m_iwlConstanceLock.WriteLock();
		if (m_mpConstances.find(strValueName) != m_mpConstances.end()) {
			m_iwlConstanceLock.WriteUnlock();
			return;
		}
		m_mpConstances.insert(_ValuePair(strValueName, ivValue));
		m_iwlConstanceLock.WriteUnlock();
	}

	inline void AddGlobalValue(const string& strValueName, const IrisValue& ivValue) {
		m_iwlGlobalVariableLock.WriteLock();
		m_mpGlobalValues.insert(_ValuePair(strValueName, ivValue));
		m_iwlGlobalVariableLock.WriteUnlock();
	}

	inline void AddOtherValue(const string& strValueName, const IrisValue& ivValue) {
		m_iwlOtherVariableLock.WriteLock();
		m_mpOtherValues.insert(_ValuePair(strValueName, ivValue));
		m_iwlOtherVariableLock.WriteUnlock();
	}

	inline const IrisValue& GetConstance(const string& strValueName, bool& bResult) {
		bResult = true;
		m_iwlConstanceLock.ReadLock();
		decltype(m_mpConstances)::iterator iCons;
		if ((iCons = m_mpConstances.find(strValueName)) == m_mpConstances.end()) {
			bResult = false;
			m_iwlConstanceLock.ReadUnlock();
			return m_ivNil;
		}
		else {
			auto& ivResult = iCons->second;
			m_iwlConstanceLock.ReadUnlock();
			return ivResult;
		}
	}

	inline const IrisValue& GetGlobalValue(const string& strValueName, bool& bResult) {
		bResult = true;
		m_iwlGlobalVariableLock.ReadLock();
		decltype(m_mpGlobalValues)::iterator iGlob;
		if ((iGlob = m_mpGlobalValues.find(strValueName)) == m_mpGlobalValues.end()) {
			bResult = false;
			m_iwlGlobalVariableLock.ReadUnlock();
			return m_ivNil;
		}
		else {
			auto& ivResult = iGlob->second;
			m_iwlGlobalVariableLock.ReadUnlock();
			return ivResult;
		}
	}

	inline const IrisValue& GetOtherValue(const string& strValueName, bool& bResult) {
		bResult = true;
		m_iwlOtherVariableLock.ReadLock();
		decltype(m_mpOtherValues)::iterator iOth;
		if ((iOth = m_mpOtherValues.find(strValueName)) == m_mpOtherValues.end()) {
			bResult = false;
			m_iwlOtherVariableLock.ReadUnlock();
			return m_ivNil;
		}
		else {
			auto& ivResult = iOth->second;
			m_iwlOtherVariableLock.ReadUnlock();
			return ivResult;
		}
	}

	inline IrisMethod* GetMainMethod(const string& strMethodName);

	inline void AddMainMethod(const string& strMethodName, IrisMethod* pMethod);


	inline const IrisValue& Nil() { return m_ivNil; }
	inline const IrisValue& True() { return m_ivTrue; }
	inline const IrisValue& False() { return m_ivFalse; }

	friend class IrisGC;

	// -------------- Instructors --------------

private:
	bool BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, string& strMethodName, unsigned int nCurrentFileIndex);
	void GetCodesFromBlock(unsigned int nIndex, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisCodeSegment& icsCodeSegment);

public:
	bool push_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cre_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool nol_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool assign(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool hid_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool set_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool clr_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool fld_load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool load_nil(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool load_true(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool load_false(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool load_self(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool imth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cmth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool blk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool end_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool jfon(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool jmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool ini_tm(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool ini_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cmp_tac(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool inc_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool assign_log(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool brk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool rtn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex, unsigned int nEnder);
	bool ctn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool assign_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool assign_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool load_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool jt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool assign_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cmp_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cre_cenv(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool def_cls(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool add_ext(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool add_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool add_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool push_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool pop_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool str_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool gtr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool gstr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool set_auth(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool def_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool def_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool def_infs(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cblk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool blk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool cast(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool reg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool ureg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool assign_ir(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool grn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);
	bool spr(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nLineNumber, unsigned int nCurrentFileIndex);

	friend class IrisKernel;
};

#endif