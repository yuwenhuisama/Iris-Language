#ifndef _H_IRISINTERPRETER_
#define _H_IRISINTERPRETER_

#include "IrisCompileConfigure.h"

#include "IrisComponents/IrisVirtualCodeStructures.h"
#include "IrisUnil/IrisValue.h"
#include "IrisUnil/IrisTree.h"
#include "IrisUnil/IrisHeap.h"
#include "IrisUnil/IrisStack.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisDevelopUtil.h"
#include "IrisUnil/IrisCodeSegment.h"
#include "IrisThread/IrisWLLock.h"

#include "IrisUnil/IrisInternString.h"

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

#if IR_USE_STL_STRING
	typedef unordered_map<string, IrisValue> _ValueMap;
	typedef pair<string, IrisValue> _ValuePair;
	typedef unordered_map<string, IrisMethod*> _MethodMap;
	typedef pair<string, IrisMethod*> _MethodPair;
#else
	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _ValueMap;
	typedef pair<IrisInternString, IrisValue> _ValuePair;
	typedef unordered_map<IrisInternString, IrisMethod*, IrisInternString::IrisInerStringHash> _MethodMap;
	typedef pair<IrisInternString, IrisMethod*> _MethodPair;
#endif // IR_USE_STL_STRING
	typedef unordered_map<string, HMODULE> _ExtentionMap;
	typedef pair<string, HMODULE> _ExtentionPair;
	
	typedef bool (*ExitConditonFunction)();

public:

#define INNER_CLASS_MEMBER_NAME(klass) m_p##klass##Class
#define INNER_CLASS_MEMBER(klass) IIrisClass* INNER_CLASS_MEMBER_NAME(klass) = nullptr
#define INNER_CLASS_GET_POINTER(klass)  IrisInterpreter::CurrentInterpreter()->m_icrInnerClassesRecord.INNER_CLASS_MEMBER_NAME(klass)
#define INNER_CLASS_SET_POINTER(klass, p) IrisInterpreter::CurrentInterpreter()->m_icrInnerClassesRecord.INNER_CLASS_MEMBER_NAME(klass) = p
	struct InterClassesRecord {
		INNER_CLASS_MEMBER(Class);
		INNER_CLASS_MEMBER(Module);
		INNER_CLASS_MEMBER(Interface);
		INNER_CLASS_MEMBER(Object);
		INNER_CLASS_MEMBER(String);
		INNER_CLASS_MEMBER(UniqueString);
		INNER_CLASS_MEMBER(Integer);
		INNER_CLASS_MEMBER(Float);
		INNER_CLASS_MEMBER(Array);
		INNER_CLASS_MEMBER(Hash);
		INNER_CLASS_MEMBER(Range);
		INNER_CLASS_MEMBER(Block);
	};

public:
	InterClassesRecord m_icrInnerClassesRecord;

private:
	static IrisInterpreter* s_pInstance;

private:

	recursive_mutex m_rmHeapInsertMT;

	IrisHeap m_hpHeap;
	//IrisStack m_stStack;

	//IrisTree<IrisModule*> m_trModuels;
	//_ClassMap m_mpClasses;
	//_InterfaceMap m_mpInterfaces;
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
#if IR_USE_STL_STRING
	int _Split(const string& str, list<string>& ret_, string sep = ",");
	IrisModule* _GetLastModuleFromPath(const list<string>& lsPath);
#else
	int _Split(const string& str, list<IrisInternString>& ret_, string sep = ",");
	IrisModule* _GetLastModuleFromPath(const list<IrisInternString>& lsPath);
#endif // IR_USE_STL_STRING

public:
	static IrisInterpreter* CurrentInterpreter();
	static void SetCurrentInterpreter(IrisInterpreter* pInter);

public:

	IrisAM GetOneAM(vector<IR_WORD>& vcVector, unsigned int& nCodePointer);

	IrisClass* GetIrisClass(const string& strClassFullFieldName);
	IrisModule* GetIrisModule(const string& strModuleFullFiledName);
	IrisInterface* GetIrisInterface(const string& strInterfaceFullFiledName);

#if IR_USE_STL_STRING
	IrisClass* GetIrisClass(const list<string>& lsRoute);
	IrisModule* GetIrisModule(const list<string>& lsRoute);
	IrisInterface* GetIrisInterface(const list<string>& lsRoute);
#else
	IrisClass* GetIrisClass(const list<IrisInternString>& lsRoute);
	IrisModule* GetIrisModule(const list<IrisInternString>& lsRoute);
	IrisInterface* GetIrisInterface(const list<IrisInternString>& lsRoute);
#endif // IR_USE_STL_STRING

	bool RegistClass(const string& strClassFullFieldName, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(const string& strModuleFullFieldName, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(const string& strInterfaceFullFieldName, IIrisInterface* pInterface, bool bNative = true);

#if IR_USE_STL_STRING
	bool RegistClass(list<string>& lsPath, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(list<string>& lsPath, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(list<string>& lsPath, IIrisInterface* pInterface, bool bNative = true);
#else
	bool RegistClass(list<IrisInternString>& lsPath, IIrisClass* pClass, bool bNative = true);
	bool RegistModule(list<IrisInternString>& lsPath, IIrisModule* pModule, bool bNative = true);
	bool RegistInterface(list<IrisInternString>& lsPath, IIrisInterface* pInterface, bool bNative = true);
#endif // IR_USE_STL_STRING

	bool AddNewInstanceToHeap(IrisValue& ivValue);
	bool AddNewEnvironmentToHeap(IrisContextEnvironment* pEnvironment, IrisThreadInfo* pThreadInfo);

	bool Initialize();
	bool ShutDown();

public:

	inline void SetCompiler(IrisCompiler* pCompiler) { m_pCurrentCompiler = pCompiler; };
	bool Run();
	//bool RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer);
	bool RunCode(vector<IR_WORD>& vcVector, unsigned int nStartPointer, unsigned int nEndPointer, IrisThreadInfo* pThreadInfo);
	inline void SetExitConditionFunction(ExitConditonFunction pfFunction) { m_pfExitConditionFunction = pfFunction; }

	bool LoadExtension(const string& strExtensionPath);

	~IrisInterpreter();

	// inlines
public:


	//inline void RegisiterClosuerBlock(IrisClosureBlock* pBlock) {
	//	m_pClosureBlockRegister = pBlock;
	//}

	void PopStack(unsigned int nTimes, IrisThreadInfo* pThreadInfo);

	inline bool FatalErrorHappened(IrisThreadInfo* pThreadInfo) { return pThreadInfo->m_bFatalErrorHappendRegister; }
	inline void HappenFatalError(IrisThreadInfo* pThreadInfo) { pThreadInfo->m_bFatalErrorHappendRegister = true; }

	inline bool IrregularHappened(IrisThreadInfo* pThreadInfo) { return pThreadInfo->m_bIrregularHappenedRegister; }
	inline void RegistIrregular(IrisValue& ivValue, IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_ivIrregularObjectRegister = ivValue;
		pThreadInfo->m_bIrregularHappenedRegister = true;
	}
	inline void UnregistIrregular(IrisThreadInfo* pThreadInfo) { pThreadInfo->m_bIrregularHappenedRegister = false; }

	inline bool ForceStop(IrisThreadInfo* pThreadInfo) { return pThreadInfo->m_bIrregularHappenedRegister; }

	inline bool PushEnvironment(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_skEnvironmentStack.push_back(pThreadInfo->m_pEnvrionmentRegister);
		return true;
	}
	inline bool SetEnvironment(IrisContextEnvironment* pEnvironment, IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_pEnvrionmentRegister = pEnvironment;
		return true;
	}
	inline bool PopEnvironment(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_pEnvrionmentRegister = pThreadInfo->m_skEnvironmentStack.back();
		pThreadInfo->m_skEnvironmentStack.pop_back();
		return true;
	}

	inline void PushVessle(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_skVessleRegister.push_back(pThreadInfo->m_ivVessleRegister);
	}
	inline void PopVessle(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_ivVessleRegister = pThreadInfo->m_skVessleRegister.back();
		pThreadInfo->m_skVessleRegister.pop_back();
	}

	inline void PushIterator(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_skIteratorRegister.push_back(pThreadInfo->m_ivIteratorRegister); }
	inline void PopIterator(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_ivIteratorRegister = pThreadInfo->m_skIteratorRegister.back();
		pThreadInfo->m_skIteratorRegister.pop_back();
	}

	inline void PushCounter(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_skCounterRegister.push_back(pThreadInfo->m_ivCounterRegister);
	}
	inline void PopCounter(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_ivCounterRegister = pThreadInfo->m_skCounterRegister.back(); 
		pThreadInfo->m_skCounterRegister.pop_back(); 
	}

	inline void PushTimer(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_skTimerRegister.push_back(pThreadInfo->m_ivTimerRegister); }
	inline void PopTimer(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_ivTimerRegister = pThreadInfo->m_skTimerRegister.back();
		pThreadInfo->m_skTimerRegister.pop_back();
	}

	inline void PushUnlimitedLoopFlag(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_skUnimitedLoopFlag.push_back(pThreadInfo->m_bUnimitedLoopFlagRegister); }
	inline void PopUnlimitedLoopFlag(IrisThreadInfo* pThreadInfo) {
		pThreadInfo->m_bUnimitedLoopFlagRegister = pThreadInfo->m_skUnimitedLoopFlag.back();
		pThreadInfo->m_skUnimitedLoopFlag.pop_back(); 
	}

	inline unsigned int GetTopDeepIndex(IrisThreadInfo* pThreadInfo) {
		return pThreadInfo->m_skDeepStack.empty() ? -1 : pThreadInfo->m_skDeepStack.back(); 
	}
	inline void PushDeepIndex(unsigned int nIndex, IrisThreadInfo* pThreadInfo) { pThreadInfo->m_skDeepStack.push_back(nIndex); }
	inline void PopTopDeepIndex(IrisThreadInfo* pThreadInfo) { pThreadInfo->m_skDeepStack.pop_back(); }
	inline void ClearDeepStack(IrisThreadInfo* pThreadInfo) { pThreadInfo->m_skDeepStack.clear(); }

	inline unsigned int GetTopMethodDeepIndex(IrisThreadInfo* pThreadInfo) { return pThreadInfo->m_skMethodDeepStack.back(); }
	inline void PushMethodDeepIndex(unsigned int nIndex, IrisThreadInfo* pThreadInfo) { pThreadInfo->m_skMethodDeepStack.push_back(nIndex); }
	inline void PopMethodTopDeepIndex(IrisThreadInfo* pThreadInfo) { pThreadInfo->m_skMethodDeepStack.pop_back(); }
	inline void ClearMethodDeepStack(IrisThreadInfo* pThreadInfo) { pThreadInfo->m_skMethodDeepStack.clear(); }

	inline const IrisValue& GetCurrentResultRegister(IrisThreadInfo* pThreadInfo) { return pThreadInfo->m_ivResultRegister; }

	inline IrisContextEnvironment* GetCurrentContextEnvrionment(IrisThreadInfo* pThreadInfo) {
		return pThreadInfo->m_pEnvrionmentRegister;
	}

#if IR_USE_STL_STRING
	inline void AddConstance(const string& strValueName, const IrisValue& ivValue) {
#else
	inline void AddConstance(const IrisInternString& strValueName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
		m_iwlConstanceLock.WriteLock();
		if (m_mpConstances.find(strValueName) != m_mpConstances.end()) {
			m_iwlConstanceLock.WriteUnlock();
			return;
		}
		m_mpConstances.insert(_ValuePair(strValueName, ivValue));
		m_iwlConstanceLock.WriteUnlock();
	}

#if IR_USE_STL_STRING
	inline void AddGlobalValue(const string& strValueName, const IrisValue& ivValue) {
#else
	inline void AddGlobalValue(const IrisInternString& strValueName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
		m_iwlGlobalVariableLock.WriteLock();
		m_mpGlobalValues.insert(_ValuePair(strValueName, ivValue));
		m_iwlGlobalVariableLock.WriteUnlock();
	}

#if IR_USE_STL_STRING
	inline void AddOtherValue(const string& strValueName, const IrisValue& ivValue) {
#else
	inline void AddOtherValue(const IrisInternString& strValueName, const IrisValue& ivValue) {
#endif // IR_USE_STL_STRING
		m_iwlOtherVariableLock.WriteLock();
		m_mpOtherValues.insert(_ValuePair(strValueName, ivValue));
		m_iwlOtherVariableLock.WriteUnlock();
	}

#if IR_USE_STL_STRING
	inline const IrisValue& GetConstance(const string& strValueName, bool& bResult) {
#else
	inline const IrisValue& GetConstance(const IrisInternString& strValueName, bool& bResult) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
	inline const IrisValue& GetGlobalValue(const string& strValueName, bool& bResult) {
#else
	inline const IrisValue& GetGlobalValue(const IrisInternString& strValueName, bool& bResult) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
	inline const IrisValue& GetOtherValue(const string& strValueName, bool& bResult) {
#else
	inline const IrisValue& GetOtherValue(const IrisInternString& strValueName, bool& bResult) {
#endif // IR_USE_STL_STRING
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

#if IR_USE_STL_STRING
	inline IrisMethod* GetMainMethod(const string& strMethodName);
#else
	inline IrisMethod* GetMainMethod(const IrisInternString& strMethodName);
#endif // IR_USE_STL_STRING

#if IR_USE_STL_STRING
	inline void AddMainMethod(const string& strMethodName, IrisMethod* pMethod);
#else
	inline void AddMainMethod(const IrisInternString& strMethodName, IrisMethod* pMethod);
#endif // IR_USE_STL_STRING


	inline const IrisValue& Nil() { return m_ivNil; }
	inline const IrisValue& True() { return m_ivTrue; }
	inline const IrisValue& False() { return m_ivFalse; }

	friend class IrisGC;

	// -------------- Instructors --------------

private:
#if IR_USE_STL_STRING
	bool BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, string& strMethodName, IrisThreadInfo* pThreadInfo);
#else
	bool BuildUserFunction(void** pFunction, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisInternString& strMethodName, IrisThreadInfo* pThreadInfo);
#endif // IR_USE_STL_STRING
	void GetCodesFromBlock(unsigned int nIndex, vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisCodeSegment& icsCodeSegment);

	// virtual bytecode
public:
	bool push_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool cre_env(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool nol_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool assign(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool hid_call(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool set_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool clr_fld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool fld_load(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load_nil(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load_true(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load_false(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load_self(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool imth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool cmth_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool blk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool end_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool jfon(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool jmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool ini_tm(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool ini_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool cmp_tac(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool inc_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool assign_log(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool brk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_deep(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool rtn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, unsigned int nEnder, IrisThreadInfo* pThreadInfo);
	bool ctn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool assign_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool assign_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool jt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool assign_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool cmp_cmp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool cre_cenv(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool def_cls(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool add_ext(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool add_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool add_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_cnt(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_tim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_unim(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_vsl(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool push_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool pop_iter(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool str_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool gtr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool gstr_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool set_auth(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool def_mld(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool def_inf(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool def_infs(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool cblk_def(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool blk(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	//bool cast(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool reg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool ureg_irp(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool assign_ir(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool grn(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool spr(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);
	bool load_cast(vector<IR_WORD>& vcVector, unsigned int& nCodePointer, IrisThreadInfo* pThreadInfo);

	friend class IrisKernel;
};

#endif